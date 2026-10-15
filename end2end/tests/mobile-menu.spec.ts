import { test, expect, devices } from "@playwright/test";

const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? "http://localhost:3000";

test.use({ ...devices["Pixel 5"] });

for (const javaScriptEnabled of [true, false]) {
  test.describe(`Pixel menu with JavaScript ${javaScriptEnabled ? "enabled" : "disabled"}`, () => {
    test.use({ javaScriptEnabled });

    test("opens, closes, and navigates by touch", async ({ page }) => {
      await page.goto(`${baseURL}/`);

      const menu = page.locator("#mobileMenu");
      const toggle = page.locator('summary[aria-controls="mobileMenu"]');
      const disclosure = page.locator("details.mobile-navigation");
      await expect(menu).toBeHidden();
      await toggle.tap();
      await expect(menu).toBeVisible();
      await expect(disclosure).toHaveAttribute("open", "");

      await toggle.tap();
      await expect(menu).toBeHidden();

      await toggle.tap();
      await menu.getByRole("link", { name: "Blog", exact: true }).tap();
      await expect(page).toHaveURL(`${baseURL}/blog`);
      await expect(menu).toBeHidden();
    });
  });
}
