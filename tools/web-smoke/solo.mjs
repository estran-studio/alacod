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
const context = await browser.newContext({
  viewport: { width: 1200, height: 950 },
});
const page = await context.newPage();
const origin = process.env.SITE_ORIGIN || "http://127.0.0.1:4174";
let serverRequests = 0;
page.on("request", (request) => {
  if (new URL(request.url()).hostname === "allumette.estran.studio")
    serverRequests++;
});
const errors = [];
const failed = [];
page.on("pageerror", (error) => errors.push(error.message));
page.on("response", (response) => {
  if (response.status() >= 400)
    failed.push(`${response.status()} ${response.url()}`);
});
page.on("console", (message) => {
  if (message.type() === "error")
    console.log("CONSOLE", message.text().slice(0, 800));
});
for (const game of ["zombies", "throne"]) {
  await page.goto(`${origin}/play?id=${game}`, {
    waitUntil: "domcontentloaded",
  });
  await page.getByRole("button", { name: "Lancer la partie" }).click();
  try {
    await page
      .getByRole("status")
      .filter({ hasText: "Jeu lancé" })
      .waitFor({ timeout: 120000 });
    console.log(game, "initialized");
    await page.waitForTimeout(15000);
    const frame = page.frameLocator("iframe");
    await frame.locator("canvas").click({ position: { x: 300, y: 250 } });
    await page.keyboard.press("KeyR");
    await page.waitForTimeout(1500);
    await page.keyboard.down("KeyD");
    await page.waitForTimeout(1200);
    await page.keyboard.up("KeyD");
    await page.screenshot({
      path: `/tmp/alacod-${game}-browser.png`,
      fullPage: true,
    });
    console.log(game, await page.getByRole("status").innerText());
  } catch (e) {
    process.exitCode = 1;
    console.log(
      game,
      e.message,
      "alerts:",
      await page.getByRole("alert").allTextContents(),
    );
  }
}
console.log("pageerrors:", errors);
console.log("failed requests:", failed);
await context.close();
await browser.close();
console.log("Allumette requests during solo:", serverRequests);
if (errors.length || failed.length || serverRequests) process.exitCode = 1;
