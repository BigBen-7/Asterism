import base from "./packages/config/eslint.mjs";

export default [
  { ignores: ["**/dist/**", "**/target/**", "**/node_modules/**", "sandbox/**", "infra/**"] },
  ...base,
];
