import { test, expect } from "@playwright/test";

test("homepage has title and heading text", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  await expect(page).toHaveTitle("Emerald Show Blog");

  await expect(page.getByText("Welcome to the Emerald Show")).toBeVisible();
});

test("admin editor hydrates with dynamic routing", async ({ page }) => {
  await page.goto("http://localhost:3000/admin/post");

  await expect(page.locator("leptos-island")).toHaveCount(0);

  await page.getByRole("button", { name: "Display View" }).click();
  await expect(
    page.getByRole("button", { name: "Source View" }),
  ).toBeVisible();
});
