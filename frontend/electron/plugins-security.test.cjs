const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const { unsafeRendererPluginsEnabled } = require("./main/plugins.cjs");

test("renderer plugin loading stays opt-in", () => {
  assert.equal(unsafeRendererPluginsEnabled({}), false);
  assert.equal(
    unsafeRendererPluginsEnabled({ TAMUX_ENABLE_UNSAFE_RENDERER_PLUGINS: "0" }),
    false,
  );
  assert.equal(
    unsafeRendererPluginsEnabled({ TAMUX_ENABLE_UNSAFE_RENDERER_PLUGINS: "1" }),
    true,
  );
});

test("browser window keeps Electron sandbox enabled", () => {
  const source = fs.readFileSync(
    path.join(__dirname, "main", "window-runtime.cjs"),
    "utf8",
  );

  assert.match(source, /sandbox:\s*true/);
});
