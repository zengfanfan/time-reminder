declare const process: {
  env: Record<string, string | undefined>;
};

declare module "node:assert/strict" {
  interface Assert {
    equal(actual: unknown, expected: unknown, message?: string): void;
    deepEqual(actual: unknown, expected: unknown, message?: string): void;
    match(actual: string, expected: RegExp, message?: string): void;
    ok(value: unknown, message?: string): void;
  }

  const assert: Assert;
  export default assert;
}

declare module "node:test" {
  export interface TestContext {
    beforeEach(fn: () => void | Promise<void>): void;
  }

  export function test(
    name: string,
    fn: (context: TestContext) => void | Promise<void>,
  ): void;

  export const mock: {
    module(
      specifier: string,
      options: { exports: Record<string, unknown> },
    ): void;
  };
}
