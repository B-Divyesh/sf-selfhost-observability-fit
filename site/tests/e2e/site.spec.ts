import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("landing page and local sample flow work without console errors", async ({ page }) => {
  const errors: string[] = [];
  page.on("console", (message) => { if (message.type() === "error") errors.push(message.text()); });
  await page.goto("/");
  await expect(page).toHaveTitle(/Observability Fit Check/);
  await expect(page.locator("h1")).toHaveCount(1);
  await expect(page.getByRole("main")).toBeVisible();
  await page.getByRole("button", { name: "Load synthetic specimen" }).click();
  await page.getByRole("button", { name: /Inspect sample/ }).click();
  await expect(page.getByText("Your workload habitat")).toBeVisible();
  await expect(page.getByRole("cell", { name: /Good fit · all signals/ })).toBeVisible();
  expect(errors).toEqual([]);
});

test("keyboard path and empty error are usable", async ({ page }) => {
  await page.goto("/#specimen");
  await page.getByRole("button", { name: /Inspect sample/ }).focus();
  await page.keyboard.press("Enter");
  await expect(page.getByText(/Choose a sample or load/)).toBeVisible();
  await expect(page.locator("#sample-file")).toBeFocused();
});

test("rendered results have no serious or critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Load synthetic specimen" }).click();
  await page.getByRole("button", { name: /Inspect sample/ }).click();
  await expect(page.locator(".signal-mix")).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  const severe = results.violations.filter((violation) => ["serious", "critical"].includes(violation.impact ?? ""));
  expect(severe).toEqual([]);
});

test("mobile touch targets meet the 44px minimum", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== "mobile-390", "Touch-target sizing is checked at the required 390px viewport.");
  await page.goto("/");
  const targets = page.locator("header .wordmark, footer .wordmark, .copy-button, footer > div a");
  const boxes = await targets.evaluateAll((elements) => elements.map((element) => {
    const { width, height } = element.getBoundingClientRect();
    return { width, height };
  }));
  expect(boxes).toHaveLength(6);
  for (const box of boxes) {
    expect(box.width).toBeGreaterThanOrEqual(44);
    expect(box.height).toBeGreaterThanOrEqual(44);
  }
});
