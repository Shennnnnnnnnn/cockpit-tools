#!/usr/bin/env node

const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');

const args = process.argv.slice(2);

function readOption(name, fallback) {
  const prefix = `${name}=`;
  const inline = args.find((arg) => arg.startsWith(prefix));
  if (inline) {
    return inline.slice(prefix.length);
  }
  const index = args.indexOf(name);
  if (index >= 0 && args[index + 1] && !args[index + 1].startsWith('--')) {
    return args[index + 1];
  }
  return fallback;
}

function hasFlag(name) {
  return args.includes(name);
}

if (hasFlag('--help') || hasFlag('-h')) {
  console.log(`Usage: npm run sync-fork-build -- [options]

Sync the current branch from upstream origin, push it to the fork remote, then build the app bundle.

Options:
  --origin <name>              Upstream remote name. Default: origin
  --fork <name>                Fork remote name. Default: fork
  --branch <name>              Branch to sync. Default: current branch
  --build-command <command>    Build command. Default: npm run tauri build
  --fetch-timeout-ms <ms>      Timeout for each fetch. Default: 300000
  --allow-dirty               Allow running with local changes
  --skip-push                 Merge origin but do not push fork
  --skip-build                Merge/push only
`);
  process.exit(0);
}

function run(command, commandArgs, options = {}) {
  const label = [command, ...commandArgs].join(' ');
  console.log(`\n$ ${label}`);
  const result = spawnSync(command, commandArgs, {
    cwd: options.cwd || process.cwd(),
    encoding: options.encoding || 'utf8',
    env: process.env,
    shell: process.platform === 'win32',
    stdio: options.capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
    timeout: options.timeout,
  });

  if (result.error) {
    throw new Error(`${label} failed: ${result.error.message}`);
  }
  if (typeof result.status === 'number' && result.status !== 0) {
    const error = new Error(`${label} failed with exit code ${result.status}`);
    error.status = result.status;
    error.stdout = result.stdout || '';
    error.stderr = result.stderr || '';
    throw error;
  }
  return result.stdout ? result.stdout.trim() : '';
}

function runShell(commandLine, options = {}) {
  console.log(`\n$ ${commandLine}`);
  const result = spawnSync(commandLine, {
    cwd: options.cwd || process.cwd(),
    encoding: 'utf8',
    env: process.env,
    shell: true,
    stdio: options.capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
    timeout: options.timeout,
  });

  if (result.error) {
    throw new Error(`${commandLine} failed: ${result.error.message}`);
  }
  if (typeof result.status === 'number' && result.status !== 0) {
    const error = new Error(`${commandLine} failed with exit code ${result.status}`);
    error.status = result.status;
    error.stdout = result.stdout || '';
    error.stderr = result.stderr || '';
    throw error;
  }
  return result.stdout ? result.stdout.trim() : '';
}

function git(args, options = {}) {
  return run('git', args, options);
}

function isCleanWorktree() {
  const status = git(['status', '--porcelain'], { capture: true });
  return status.length === 0;
}

function currentBranch() {
  return git(['branch', '--show-current'], { capture: true });
}

function assertRemoteExists(remote) {
  git(['remote', 'get-url', remote], { capture: true });
}

function listArtifacts(repoRoot) {
  const artifactDirs = [
    path.join(repoRoot, 'target/release/bundle/macos'),
    path.join(repoRoot, 'target/release/bundle/dmg'),
  ];
  const artifacts = [];
  for (const dir of artifactDirs) {
    if (!fs.existsSync(dir)) {
      continue;
    }
    for (const name of fs.readdirSync(dir)) {
      if (name.endsWith('.app') || name.endsWith('.dmg') || name.endsWith('.tar.gz')) {
        artifacts.push(path.join(dir, name));
      }
    }
  }
  return artifacts;
}

function printArtifacts(repoRoot) {
  const artifacts = listArtifacts(repoRoot);
  if (artifacts.length === 0) {
    console.log('No bundle artifacts found.');
    return;
  }
  console.log('\nBundle artifacts:');
  for (const artifact of artifacts) {
    console.log(`- ${artifact}`);
  }
}

const repoRoot = git(['rev-parse', '--show-toplevel'], { capture: true });
process.chdir(repoRoot);

const originRemote = readOption('--origin', 'origin');
const forkRemote = readOption('--fork', 'fork');
const branch = readOption('--branch', currentBranch());
const buildCommand = readOption('--build-command', 'npm run tauri build');
const fetchTimeoutMs = Number(readOption('--fetch-timeout-ms', '300000'));
const allowDirty = hasFlag('--allow-dirty');
const skipPush = hasFlag('--skip-push');
const skipBuild = hasFlag('--skip-build');

if (!branch) {
  console.error('Cannot determine current branch. Pass --branch <name>.');
  process.exit(1);
}

console.log('Sync fork and build started.');
console.log(`repo: ${repoRoot}`);
console.log(`branch: ${branch}`);
console.log(`origin: ${originRemote}`);
console.log(`fork: ${forkRemote}`);

try {
  assertRemoteExists(originRemote);
  assertRemoteExists(forkRemote);

  if (!allowDirty && !isCleanWorktree()) {
    throw new Error('Working tree is not clean. Commit/stash changes or pass --allow-dirty.');
  }

  git(['fetch', originRemote, `+refs/heads/${branch}:refs/remotes/${originRemote}/${branch}`], {
    timeout: fetchTimeoutMs,
  });
  git(['fetch', forkRemote, `+refs/heads/${branch}:refs/remotes/${forkRemote}/${branch}`], {
    timeout: fetchTimeoutMs,
  });

  git(['switch', branch]);
  git(['merge', `${originRemote}/${branch}`, '--no-edit']);

  const version = JSON.parse(fs.readFileSync(path.join(repoRoot, 'package.json'), 'utf8')).version;
  console.log(`\nMerged ${originRemote}/${branch}. Current package version: ${version}`);

  if (!skipPush) {
    git(['push', forkRemote, branch]);
  }

  if (!skipBuild) {
    try {
      runShell(buildCommand, { capture: true });
    } catch (error) {
      const output = `${error.stdout || ''}\n${error.stderr || ''}`;
      const signingKeyMissing = output.includes('TAURI_SIGNING_PRIVATE_KEY');
      if (!signingKeyMissing || listArtifacts(repoRoot).length === 0) {
        process.stdout.write(error.stdout || '');
        process.stderr.write(error.stderr || '');
        throw error;
      }
      process.stdout.write(error.stdout || '');
      process.stderr.write(error.stderr || '');
      console.warn(
        '\nBuild produced bundle artifacts, but updater signing failed because TAURI_SIGNING_PRIVATE_KEY is not set.'
      );
    }
  }

  printArtifacts(repoRoot);
  console.log('\n[OK] Fork synced and build step completed.');
} catch (error) {
  console.error(`\n[FAILED] ${error.message}`);
  process.exit(error.status || 1);
}
