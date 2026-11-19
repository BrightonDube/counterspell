// ide-workspace T21 and T22 on the real stack: a keyboard-only journey from the editor through
// Submit, Ghost, Autopsy, Hint and Rerun and back, and the same failing run with and without
// reduced motion. Same prerequisites as real-api-check.mjs (see README.md).
import {
  BASE,
  createReporter,
  focusLabel,
  launchBrowser,
  openProblem,
  signIn,
  typeSource
} from './support.mjs';

const { check, finish } = createReporter();
const WRONG = 'def add_two(a, b):\n    return a * b\n';
const browser = await launchBrowser();

async function failingSubmit(page) {
  await typeSource(page, WRONG);
  await page.keyboard.press('Control+Shift+Enter');
  await page.waitForSelector('text=Wrong answer', { timeout: 180000 });
  // The Rerun button exists only once the engine's ghost event has arrived.
  await page.getByRole('button', { name: 'Rerun ghost' }).waitFor({
    timeout: 120000
  });
}

// ---------- T21: keyboard-only journey (normal motion) ----------
{
  const context = await browser.newContext();
  const page = await context.newPage();
  check('keyboard: sign-in', await signIn(context, page, 'keyboard'));
  await openProblem(page, 'add-two');

  // Submit by shortcut from inside the editor. Results arrive while the caret stays put.
  await typeSource(page, WRONG);
  await page.keyboard.press('Control+Shift+Enter');
  const focusDuring = new Set();
  const done = page
    .getByRole('button', { name: 'Rerun ghost' })
    .waitFor({ timeout: 180000 })
    .then(() => true)
    .catch(() => false);
  let finished = false;
  done.then(() => (finished = true));
  while (!finished) {
    focusDuring.add(await focusLabel(page));
    await page.waitForTimeout(150);
  }
  check(
    'keyboard: async run events never move focus out of the editor',
    focusDuring.size === 1 && focusDuring.has('editor'),
    [...focusDuring].join(' | ')
  );

  // Shortcut help is discoverable and tied to the buttons.
  const help = await page.evaluate(() => {
    const note = document.getElementById('run-shortcuts');
    const run = [...document.querySelectorAll('button')].find(
      (b) => b.textContent?.trim() === 'Run samples'
    );
    return {
      text: note?.textContent ?? '',
      linked: run?.getAttribute('aria-describedby') === 'run-shortcuts'
    };
  });
  check(
    'keyboard: shortcut help is on the page and described by the Run button',
    help.linked && /Ctrl\+Enter/.test(help.text)
  );

  // Leave the editor with Tab only (Ctrl+M switches Monaco to Tab-moves-focus), then walk
  // forward and record every stop.
  await page.keyboard.press('Control+m');
  const forward = [];
  for (let i = 0; i < 40; i++) {
    await page.keyboard.press('Tab');
    forward.push(await focusLabel(page));
  }
  const at = (label) => forward.findIndex((l) => l.startsWith(label));
  check(
    'keyboard: Tab reaches Run, Submit, Rerun ghost and Show hint 1 in reading order',
    at('Run samples') >= 0 &&
      at('Submit') > at('Run samples') &&
      at('Rerun ghost') > at('Submit') &&
      at('Show hint 1') > at('Rerun ghost'),
    forward.filter((l, i) => forward.indexOf(l) === i).join(' > ')
  );

  // Activate the hint by keyboard from where Tab left focus.
  await page.evaluate(() => {
    const hint = [...document.querySelectorAll('button')].find(
      (b) => b.textContent?.trim() === 'Show hint 1'
    );
    hint?.focus();
  });
  await page.keyboard.press('Enter');
  const hinted = await page
    .waitForSelector('text=Read the signature', { timeout: 30000 })
    .then(() => true)
    .catch(() => false);
  check('keyboard: Enter on Show hint 1 reveals the authored rung', hinted);

  // Rerun ghost with Space on the focused button.
  await page.evaluate(() => {
    const rerun = [...document.querySelectorAll('button')].find(
      (b) => b.textContent?.trim() === 'Rerun ghost'
    );
    rerun?.focus();
  });
  const rerunPosted = page
    .waitForRequest(
      (r) => r.url().endsWith('/api/runs/ghost') && r.method() === 'POST',
      { timeout: 15000 }
    )
    .then(() => true)
    .catch(() => false);
  await page.keyboard.press('Space');
  check(
    'keyboard: Space on Rerun ghost sends the custom run',
    await rerunPosted
  );

  // Back to the editor with Shift+Tab only.
  let backInEditor = false;
  for (let i = 0; i < 60 && !backInEditor; i++) {
    await page.keyboard.press('Shift+Tab');
    backInEditor = (await focusLabel(page)) === 'editor';
  }
  check('keyboard: Shift+Tab returns to the editor', backInEditor);
  await context.close();
}

// ---------- T22: reduced motion in a real browser ----------
async function motionRun(reduced) {
  const context = await browser.newContext({
    reducedMotion: reduced ? 'reduce' : 'no-preference'
  });
  const page = await context.newPage();
  const scripts = new Set();
  page.on('response', (r) => {
    if (r.url().endsWith('.js')) scripts.add(r.url());
  });
  const label = reduced ? 'reduced' : 'normal';
  if (!(await signIn(context, page, `motion-${label}`)))
    check(`${label}: sign-in`, false);
  await openProblem(page, 'add-two');

  // Count every CSS animation or transition a person could see (longer than 1 ms) that starts
  // while the failure plays out. The global reduced-motion rule shortens every transition to
  // 0.01 ms, which still fires events, so the duration is what tells motion from none.
  await page.evaluate(() => {
    window.__started = 0;
    const ms = (value) =>
      Math.max(
        ...value
          .split(',')
          .map((part) =>
            part.trim().endsWith('ms')
              ? parseFloat(part)
              : parseFloat(part) * 1000
          )
      );
    document.addEventListener(
      'transitionrun',
      (event) => {
        if (ms(getComputedStyle(event.target).transitionDuration) > 1)
          window.__started += 1;
      },
      true
    );
    document.addEventListener(
      'animationstart',
      (event) => {
        if (ms(getComputedStyle(event.target).animationDuration) > 1)
          window.__started += 1;
      },
      true
    );
  });
  await failingSubmit(page);
  await page.waitForTimeout(1500);
  const started = await page.evaluate(() => window.__started);
  const steps = await page.evaluate(
    () =>
      [...document.querySelectorAll('li')].filter((li) =>
        /^Step \d+: /.test(li.textContent?.trim() ?? '')
      ).length
  );
  // Calm IDE: once the run is over, nothing keeps animating.
  await page.waitForTimeout(3000);
  const running = await page.evaluate(
    () =>
      document.getAnimations().filter((a) => a.playState === 'running').length
  );
  let gsapLoaded = false;
  for (const url of scripts) {
    const body = await (await fetch(url)).text();
    if (/GreenSock|gsap\.version|gsap-/i.test(body)) gsapLoaded = true;
  }
  await context.close();
  return { started, steps, running, gsapLoaded };
}

const normal = await motionRun(false);
const reduced = await motionRun(true);
check(
  'motion: reduced motion starts no visible transition or animation on the failing run',
  reduced.started === 0 && normal.started > 0,
  `normal=${normal.started} reduced=${reduced.started}`
);
check(
  'motion: the shrink trace is listed in full under both settings',
  reduced.steps >= 3 && reduced.steps === normal.steps,
  `normal=${normal.steps} reduced=${reduced.steps}`
);
check(
  'motion: nothing is left animating once the run is over (no ambient loop)',
  normal.running === 0 && reduced.running === 0,
  `normal=${normal.running} reduced=${reduced.running}`
);
check(
  'motion: the GSAP chunk loads in normal motion and never under reduced motion',
  normal.gsapLoaded && !reduced.gsapLoaded,
  `normal=${normal.gsapLoaded} reduced=${reduced.gsapLoaded}`
);

await browser.close();
finish();
