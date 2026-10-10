import { chromium } from "playwright-core";
const browser = await chromium.launch({
  executablePath: process.env.CHROMIUM_EXECUTABLE || "/usr/bin/chromium",
  headless: true,
  args: [
    ...(process.env.CHROMIUM_NO_SANDBOX === "1" ? ["--no-sandbox"] : []),
    "--use-gl=angle",
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
  ],
});
const origin = process.env.SITE_ORIGIN || "http://127.0.0.1:4174";
const server =
  process.env.ALLUMETTE_ORIGIN || "https://allumette.estran.studio";
const clean = (text) => text.replace(/eyJ[A-Za-z0-9_.-]+/g, "[ticket]");
const errors = [],
  bad = [];
for (let i = 0; i < 100; i++) {
  try {
    if ((await fetch(`${origin}/online`)).ok) break;
  } catch {}
  await new Promise((r) => setTimeout(r, 200));
}
const contexts = [];
for (let i = 0; i < 2; i++) {
  const context = await browser.newContext({
    viewport: { width: 1200, height: 950 },
  });
  contexts.push(context);
  await context.addInitScript((server) => {
    localStorage.setItem(
      "matchboxSettings",
      JSON.stringify({ allumetteServerUrl: server }),
    );
    window.__rtc = [];
    const Native = window.RTCPeerConnection;
    window.RTCPeerConnection = class extends Native {
      constructor(config) {
        super({ ...config, iceTransportPolicy: "relay" });
        window.__rtc.push(this);
      }
    };
  }, server);
}
const a = await contexts[0].newPage(),
  b = await contexts[1].newPage();
for (const p of [a, b]) {
  p.on("pageerror", (e) => {
    errors.push(clean(e.message));
    console.log("page-error", clean(e.message));
  });
  p.on("response", (r) => {
    if (r.status() >= 400)
      bad.push(
        `${r.status()} ${new URL(r.url()).pathname.replace(/eyJ[^/]+/g, "[ticket]")}`,
      );
  });
  p.on("console", (m) => {
    if (
      m.text().includes("All 2 players are connected") ||
      m.text().includes("Waiting for players: 2")
    )
      console.log("ENGINE", m.text().slice(0, 180));
    if (m.text().includes("Desync detected")) errors.push(clean(m.text()));
    if (m.type() === "error")
      console.log("console-error", clean(m.text()).slice(0, 250));
  });
}
try {
  await a.goto(`${origin}/online`);
  await a.getByLabel("Pseudonyme").fill("Alice");
  await a.getByRole("button", { name: "Se connecter", exact: true }).click();
  await a
    .getByLabel("Jeu", { exact: true })
    .selectOption(process.env.SMOKE_GAME || "zombies");
  await a.getByRole("button", { name: "Créer un salon privé" }).click();
  await a.getByLabel("Lien à partager (15 minutes)").waitFor();
  const link = await a.getByLabel("Lien à partager (15 minutes)").inputValue();
  await b.goto(link);
  await b.getByLabel("Pseudonyme").fill("Bob");
  await b.getByRole("button", { name: "Se connecter", exact: true }).click();
  await a
    .getByText("2 / 2 joueurs", { exact: false })
    .waitFor({ timeout: 15000 });
  console.log("lobby: two invited guests, correct build, capacity 2");
  if ((await b.url()).includes("#"))
    throw new Error("Invitation fragment was not removed");
  await a.getByRole("button", { name: "Je suis prêt", exact: true }).click();
  await b.getByRole("button", { name: "Je suis prêt", exact: true }).click();

  await a
    .getByRole("button", { name: "Lancer la partie", exact: true })
    .click();
  await Promise.all([
    a.getByRole("status").waitFor({ timeout: 120000 }),
    b.getByRole("status").waitFor({ timeout: 120000 }),
  ]);
  await Promise.all([
    a
      .getByRole("status")
      .filter({ hasText: /Jeu chargé|Connexion établie/ })
      .waitFor({ timeout: 120000 }),
    b
      .getByRole("status")
      .filter({ hasText: /Jeu chargé|Connexion établie/ })
      .waitFor({ timeout: 120000 }),
  ]);
  console.log("both WASM initialized");
  await a.waitForTimeout(15000);
  for (const p of [a, b]) {
    const f = p.frames().find((f) => f.parentFrame() === p.mainFrame());
    console.log(
      "RTC checkpoint",
      await f.evaluate(() =>
        window.__rtc.map((pc) => ({
          connection: pc.connectionState,
          ice: pc.iceConnectionState,
          gathering: pc.iceGatheringState,
          signaling: pc.signalingState,
          localType: pc.localDescription?.type,
          remoteType: pc.remoteDescription?.type,
          localCandidates: (
            pc.localDescription?.sdp.match(/a=candidate:/g) || []
          ).length,
          remoteCandidates: (
            pc.remoteDescription?.sdp.match(/a=candidate:/g) || []
          ).length,
        })),
      ),
    );
  }
  for (const p of [a, b]) {
    const f = p.frames().find((f) => f.parentFrame() === p.mainFrame());
    try {
      await f.waitForFunction(
        () => window.__rtc.some((pc) => pc.connectionState === "connected"),
        {},
        { timeout: 90000 },
      );
    } catch (e) {
      console.log(
        "RTC diagnostics",
        await f.evaluate(() =>
          window.__rtc.map((pc) => ({
            connection: pc.connectionState,
            ice: pc.iceConnectionState,
            gathering: pc.iceGatheringState,
            signaling: pc.signalingState,
          })),
        ),
      );
      throw e;
    }
  }
  console.log("both WebRTC connections established");
  for (const p of [a, b]) {
    const f = p.frames().find((f) => f.parentFrame() === p.mainFrame());
    console.log(
      "selected candidates",
      await f.evaluate(async () => {
        const pc = window.__rtc.find((p) => p.connectionState === "connected");
        const s = await pc.getStats();
        const t = [...s.values()].find(
          (x) => x.type === "transport" && x.selectedCandidatePairId,
        );
        const pair = t && s.get(t.selectedCandidatePairId);
        return {
          localType: pair && s.get(pair.localCandidateId)?.candidateType,
          remoteType: pair && s.get(pair.remoteCandidateId)?.candidateType,
          bytesReceived: pair?.bytesReceived,
        };
      }),
    );
  }
  await a.waitForTimeout(6000);
  await a.frameLocator("iframe").locator("canvas").focus();
  await a.keyboard.down("KeyD");
  await a.waitForTimeout(750);
  await a.keyboard.up("KeyD");
  await b.frameLocator("iframe").locator("canvas").focus();
  await b.keyboard.down("KeyA");
  await b.waitForTimeout(750);
  await b.keyboard.up("KeyA");
  await a.waitForTimeout(2000);
  await a.screenshot({
    path: `/tmp/alacod-online-${process.env.SMOKE_GAME || "zombies"}-alice.png`,
    fullPage: true,
  });
  await b.screenshot({
    path: `/tmp/alacod-online-${process.env.SMOKE_GAME || "zombies"}-bob.png`,
    fullPage: true,
  });
  console.log(
    "canvas participants:",
    await a
      .frameLocator("iframe")
      .locator("canvas")
      .getAttribute("data-participants")
      .then((s) =>
        JSON.parse(s).map((p) => ({
          username: p.username,
          hasPeer: !!p.peer_id,
        })),
      ),
  );
  await a.getByRole("button", { name: "Retour au salon" }).click();
  await b
    .getByRole("button", { name: "Je suis prêt", exact: true })
    .waitFor({ timeout: 15000 });
  console.log("return to lobby: both peers reset");
  await a.getByRole("button", { name: "Je suis prêt", exact: true }).click();
  await b.getByRole("button", { name: "Je suis prêt", exact: true }).click();
  await a
    .getByRole("button", { name: "Lancer la partie", exact: true })
    .click();
  await Promise.all([
    a
      .getByRole("status")
      .filter({ hasText: /Jeu chargé|Connexion établie/ })
      .waitFor({ timeout: 120000 }),
    b
      .getByRole("status")
      .filter({ hasText: /Jeu chargé|Connexion établie/ })
      .waitFor({ timeout: 120000 }),
  ]);
  for (const p of [a, b]) {
    const f = p.frames().find((f) => f.parentFrame() === p.mainFrame());
    await f.waitForFunction(
      () => window.__rtc.some((pc) => pc.connectionState === "connected"),
      {},
      { timeout: 90000 },
    );
  }
  await a.waitForTimeout(6000);
  console.log("second session connected");
  await a.getByRole("button", { name: "Retour au salon" }).click();
  await a.getByRole("button", { name: "Fermer le salon" }).click();
  await a.waitForTimeout(2000);
  console.log("errors:", errors, "failed responses:", bad);
} catch (e) {
  console.log("TEST FAILED", e.message);
  await a.screenshot({
    path: "/tmp/alacod-online-failure.png",
    fullPage: true,
  });
  console.log(
    "alerts:",
    await a.getByRole("alert").allTextContents(),
    await b.getByRole("alert").allTextContents(),
  );
  console.log("errors:", errors, "failed responses:", bad);
  process.exitCode = 1;
}
await browser.close();
if (errors.length || bad.length) process.exitCode = 1;
