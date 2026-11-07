import { test, expect, type Page } from "@playwright/test";

// The browser exercises the real Svelte UI. Only native IPC is replaced;
// Rust integration tests separately execute the actual bundled encoder.
async function desktop(page: Page, failFirst = false) {
  await page.addInitScript(
    ({ failFirst }) => {
      let callbackId = 0;
      let attempts = 0;
      const calls: { cmd: string; args: Record<string, unknown> }[] = [];
      Object.assign(window, {
        isTauri: true,
        testCalls: calls,
        __TAURI_INTERNALS__: {
          metadata: {
            currentWindow: { label: "main" },
            currentWebview: { label: "main" },
          },
          transformCallback: () => ++callbackId,
          invoke: async (cmd: string, args: Record<string, unknown> = {}) => {
            calls.push({ cmd, args });
            if (cmd === "encoder_version") return "1.6.0";
            if (cmd === "plugin:dialog|open") {
              const options = args.options as { directory?: boolean };
              return options.directory
                ? "C:\\output"
                : ["C:\\images\\photo.png", "C:\\images\\art.svg"];
            }
            if (cmd === "inspect_files")
              return (args.paths as string[]).map((path) => ({
                path,
                name: path.split("\\").pop(),
                bytes: 102400,
                extension: path.split(".").pop(),
                warning: null,
                error: null,
              }));
            if (cmd === "validate_settings") {
              const settings = args.settings as {
                targetSizeKb: number;
                targetPsnr: number;
              };
              if (settings.targetSizeKb > 0 && settings.targetPsnr > 0)
                throw "Choose either target size or target PSNR.";
            }
            if (cmd === "convert_image") {
              await new Promise((resolve) => setTimeout(resolve, 450));
              if (failFirst && attempts++ === 0)
                throw "Fixture encoder failure";
              return {
                outputPath: `${args.inputPath}.webp`,
                inputBytes: 102400,
                outputBytes: 25600,
                elapsedMs: 450,
                warning: null,
              };
            }
            return callbackId;
          },
        },
      });
    },
    { failFirst },
  );
  await page.goto("/");
  await expect(page.getByText(/cwebp 1.6.0/)).toBeVisible();
}

test("empty workspace, saved settings and responsive layout", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await desktop(page);
  await expect(
    page.getByRole("button", { name: /Convert images/ }),
  ).toBeDisabled();
  await page
    .getByRole("spinbutton", { name: "Image quality value", exact: true })
    .fill("88");
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          JSON.parse(localStorage.getItem("anything-to-webp.settings.v1")!)
            .quality,
      ),
    )
    .toBe(88);
  await page.reload();
  await expect(
    page.getByRole("spinbutton", { name: "Image quality value", exact: true }),
  ).toHaveValue("88");
  await page.screenshot({ path: "test-results/workspace.png", fullPage: true });
  await page.setViewportSize({ width: 900, height: 640 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.getByRole("tab", { name: "image", exact: true }).click();
  await page.getByLabel("Enable crop").check();
  await expect(page.getByLabel("Crop width")).toBeVisible();
  await page.getByRole("button", { name: "Reset all settings" }).click();
  await expect(page.getByLabel("Enable crop")).not.toBeChecked();
  expect(errors).toEqual([]);
});

test("deduplicates files and completes a batch with selected output and settings", async ({
  page,
}) => {
  await desktop(page);
  await page.getByRole("button", { name: "Add images", exact: true }).click();
  await page.getByRole("button", { name: "Add images", exact: true }).click();
  await expect(page.getByRole("listitem")).toHaveCount(2);
  await page.getByRole("button", { name: "Browse", exact: true }).click();
  await page.getByRole("button", { name: "High quality", exact: true }).click();
  await page
    .getByRole("button", { name: "Convert 2 images", exact: true })
    .click();
  await expect(page.getByText("Complete", { exact: true })).toHaveCount(2);
  await expect(
    page.getByRole("button", { name: "Show in folder" }),
  ).toHaveCount(2);
  await expect(page.getByRole("status")).toContainText("150.0 KB saved");
  const conversions = await page.evaluate(() =>
    (
      window as unknown as {
        testCalls: {
          cmd: string;
          args: { outputDirectory: string; settings: { quality: number } };
        }[];
      }
    ).testCalls.filter((call) => call.cmd === "convert_image"),
  );
  expect(conversions).toHaveLength(2);
  expect(conversions[0].args.outputDirectory).toBe("C:\\output");
  expect(conversions[0].args.settings.quality).toBe(92);
  await page.screenshot({
    path: "test-results/completed-batch.png",
    fullPage: true,
  });
});

test("stops after the current image and resumes remaining images", async ({
  page,
}) => {
  await desktop(page);
  await page.getByRole("button", { name: "Add images", exact: true }).click();
  await page
    .getByRole("button", { name: "Convert 2 images", exact: true })
    .click();
  await page.getByRole("button", { name: "Stop after current image" }).click();
  await expect(page.getByText("Complete", { exact: true })).toHaveCount(1);
  await expect(
    page.getByRole("button", { name: "Convert 1 image", exact: true }),
  ).toBeEnabled();
  await page
    .getByRole("button", { name: "Convert 1 image", exact: true })
    .click();
  await expect(page.getByText("Complete", { exact: true })).toHaveCount(2);
});

test("continues after a failed image and retries only failed jobs", async ({
  page,
}) => {
  await desktop(page, true);
  await page.getByRole("button", { name: "Add images", exact: true }).click();
  await page
    .getByRole("button", { name: "Convert 2 images", exact: true })
    .click();
  await expect(page.getByText("Fixture encoder failure")).toBeVisible();
  await expect(page.getByText("Complete", { exact: true })).toHaveCount(1);
  await page.getByRole("button", { name: "Retry failed" }).click();
  await page
    .getByRole("button", { name: "Convert 1 image", exact: true })
    .click();
  await expect(page.getByText("Complete", { exact: true })).toHaveCount(2);
});
