# swc-plugin-enum-to-obj

An SWC plugin to convert TypeScript enums into plain objects!

## Installation

Install with your favorite package manager as devDependency.

```bash
npm i -D swc-plugin-enum-to-obj
```

Add plugin to wherever you have an SWC config (e.g. `.swcrc` file, `swc-loader` config, etc).

This plugin currently has no configuration. However you have to leave an empty object to meet SWC's API schema.

```js
{
  jsc: {
    parser: {
      syntax: 'typescript',
      tsx: true,
    },
    experimental: {
      plugins: [
        ['swc-plugin-enum-to-obj', {}],
      ],
    }
  },
}
```

## Motivation

TypeScript enums are useful if they are used in suitable ways. However the compiled code of enums makes trouble.
In order to support enum merging, **TypeScript compiles them into IIFEs**.
I bet most of you won't use this feature, but it pollutes our compiled code.

```ts
// Before
export enum SimpleNumber {
  Bar,
  Baz,
}

// TypeScript compiles the above code to:
export var SimpleNumber;
(function(SimpleNumber) {
    SimpleNumber[SimpleNumber["Bar"] = 0] = "Bar";
    SimpleNumber[SimpleNumber["Baz"] = 1] = "Baz";
})(SimpleNumber || (SimpleNumber = {}));
```

This indeed works. But when you are trying to make a library containing enums,
you will notice that **enums are *NOT* tree-shakeable**
(e.g.
[rollup](https://rollupjs.org/repl/?version=3.28.1&shareable=JTdCJTIyZXhhbXBsZSUyMiUzQW51bGwlMkMlMjJtb2R1bGVzJTIyJTNBJTVCJTdCJTIyY29kZSUyMiUzQSUyMmltcG9ydCUyMCU3QiUyMFBJJTIwJTdEJTIwZnJvbSUyMCcuJTJGZW51bS5qcyclM0IlNUNuJTVDbmNvbnNvbGUubG9nKFBJKSUzQiUyMiUyQyUyMmlzRW50cnklMjIlM0F0cnVlJTJDJTIybmFtZSUyMiUzQSUyMm1haW4uanMlMjIlN0QlMkMlN0IlMjJjb2RlJTIyJTNBJTIyZXhwb3J0JTIwdmFyJTIwU2ltcGxlTnVtYmVyJTNCJTVDbihmdW5jdGlvbihTaW1wbGVOdW1iZXIpJTIwJTdCJTVDbiUyMCUyMCUyMCUyMFNpbXBsZU51bWJlciU1QlNpbXBsZU51bWJlciU1QiU1QyUyMkJhciU1QyUyMiU1RCUyMCUzRCUyMDAlNUQlMjAlM0QlMjAlNUMlMjJCYXIlNUMlMjIlM0IlNUNuJTIwJTIwJTIwJTIwU2ltcGxlTnVtYmVyJTVCU2ltcGxlTnVtYmVyJTVCJTVDJTIyQmF6JTVDJTIyJTVEJTIwJTNEJTIwMSU1RCUyMCUzRCUyMCU1QyUyMkJheiU1QyUyMiUzQiU1Q24lN0QpKFNpbXBsZU51bWJlciUyMCU3QyU3QyUyMChTaW1wbGVOdW1iZXIlMjAlM0QlMjAlN0IlN0QpKSUzQiU1Q24lNUNuZXhwb3J0JTIwdmFyJTIwUEklMjAlM0QlMjAzLjE0JTNCJTIyJTJDJTIyaXNFbnRyeSUyMiUzQWZhbHNlJTJDJTIybmFtZSUyMiUzQSUyMmVudW0uanMlMjIlN0QlNUQlMkMlMjJvcHRpb25zJTIyJTNBJTdCJTIyb3V0cHV0JTIyJTNBJTdCJTIyZm9ybWF0JTIyJTNBJTIyZXMlMjIlN0QlMkMlMjJ0cmVlc2hha2UlMjIlM0ElMjJzbWFsbGVzdCUyMiU3RCU3RA==),
[SWC minify](https://play.swc.rs/?version=1.3.74&code=H4sIAAAAAAAAAytLLFIIzswtyEn1K81NSi2y5tJIK81LLsnMz9NAFtdUqOZSAAJksWgUjpJTYpFSrIKtggGIAPOsCWupAmsxhGqpAmqp1USxWKGmRgFVwFahulZT05qLqwzo9gBPIN9Yz9AEyE%2FOzyvOz0nVy8lP1wjw1LQGALloC6DcAAAA&config=H4sIAAAAAAAAA41VzW7bMAy%2B9ykCn3vYDhuGnYfd9gyCIlGOOlk0RCqNUeTdRzt26tZ0sEsQ8%2BM%2FP1JvT4dD80Ku%2BXl4k7%2Fy0dtCUO7fIqEhs72IpOGhB3Il9tw8LyjTCAWbCCbR9YY0XGymgKVbu0rQWjf8AofFMo5RuNRPdra0wGM0oG9zmCYhEszas6iLOYZh7dxh1xcgWslEKu5qB5lpyfJ5jRV8pQ9%2BJ%2FkRMYGkv48YSyZmhhaK5thhSrYnMGdbFC9jprZEQi3ECFYGb%2FqCvYpnHzlilphb1IP1xqEHBYoFHMczaGYSS8wySXlKPRPs4VjbFsrWGs42VctKTLhMI5FsFa8njMQm1Ky18Abu9OAGzs39bBmDKcC15K3dC8a8M5O%2FANKBZImy7UDzO2kE4dOedXhoGXMQyvKg4EJvrcos28JgYgxKZ8fOQOGoTbOArw7GzjotnRneaR9FDwZCEK4oruk1sjtpQcfjgEEBZL42aKy6Aea%2BhTv4uBAP4N9SJesEmzU6y6d9lIbuiOlBgA74hP6BgoyCcR8uciUu%2FT5eswehBnhVpdIEbI%2BALACjSdOt3HBD1kM8mjbh8f1MzArX%2B%2B3ubG6nfV%2FdS8Y%2BwRnSHo3%2FY0UeopLbeWT1stmb7YUfGimtLF78%2BuXDUyOlPC2%2FU1FNh76uCpo4eXtKvjfvSsvLsXStifRnMZz6dP0HetcFbhcHAAA%3D)).

The compiled IIFE is a side-effect function call as the view of static code analysis.
It *cannot* be safely removed because the analyzer don't have an idea about what it is doing.

Now that we know it is constructing enum objects, we can do this for it.
This plugin picks up these *compiled* enum IIFEs and convert them to object literals.

After converting, bundlers and minifiers will be pleasure to remove all unused object literals for you.
(e.g.
[rollup](https://rollupjs.org/repl/?version=3.28.1&shareable=JTdCJTIyZXhhbXBsZSUyMiUzQW51bGwlMkMlMjJtb2R1bGVzJTIyJTNBJTVCJTdCJTIyY29kZSUyMiUzQSUyMmltcG9ydCUyMCU3QiUyMFBJJTIwJTdEJTIwZnJvbSUyMCcuJTJGZW51bS5qcyclM0IlNUNuJTVDbmNvbnNvbGUubG9nKFBJKSUzQiUyMiUyQyUyMmlzRW50cnklMjIlM0F0cnVlJTJDJTIybmFtZSUyMiUzQSUyMm1haW4uanMlMjIlN0QlMkMlN0IlMjJjb2RlJTIyJTNBJTIyZXhwb3J0JTIwdmFyJTIwU2ltcGxlTnVtYmVyJTIwJTNEJTIwJTdCJTVDbiUyMCUyMCU1QyUyMkJhciU1QyUyMiUzQSUyMDAlMkMlNUNuJTIwJTIwMCUzQSUyMCU1QyUyMkJhciU1QyUyMiUyQyU1Q24lMjAlMjAlNUMlMjJCYXolNUMlMjIlM0ElMjAxJTJDJTVDbiUyMCUyMDElM0ElMjAlNUMlMjJCYXolNUMlMjIlNUNuJTdEJTNCJTVDbiU1Q25leHBvcnQlMjB2YXIlMjBQSSUyMCUzRCUyMDMuMTQlM0IlMjIlMkMlMjJpc0VudHJ5JTIyJTNBZmFsc2UlMkMlMjJuYW1lJTIyJTNBJTIyZW51bS5qcyUyMiU3RCU1RCUyQyUyMm9wdGlvbnMlMjIlM0ElN0IlMjJvdXRwdXQlMjIlM0ElN0IlMjJmb3JtYXQlMjIlM0ElMjJlcyUyMiU3RCUyQyUyMnRyZWVzaGFrZSUyMiUzQSUyMnNtYWxsZXN0JTIyJTdEJTdE),
[SWC minify](https://play.swc.rs/?version=1.3.74&code=H4sIAAAAAAAAAytLLFIIzswtyEn1K81NSi1SsFWo5lJQUHJKLFKyUjDQAbINrCBcHYh4FVDcEMQ2tIJwuWqtubjKgOYEeAJ1G%2BsZmgD5yfl5xfk5qXo5%2BekaAZ6a1gAmzu4IaAAAAA%3D%3D&config=H4sIAAAAAAAAA41VzW7bMAy%2B9ykCn3vYDhuGnYfd9gyCIlGOOlk0RCqNUeTdRzt26tZ0sEsQ8%2BM%2FP1JvT4dD80Ku%2BXl4k7%2Fy0dtCUO7fIqEhs72IpOGhB3Il9tw8LyjTCAWbCCbR9YY0XGymgKVbu0rQWjf8AofFMo5RuNRPdra0wGM0oG9zmCYhEszas6iLOYZh7dxh1xcgWslEKu5qB5lpyfJ5jRV8pQ9%2BJ%2FkRMYGkv48YSyZmhhaK5thhSrYnMGdbFC9jprZEQi3ECFYGb%2FqCvYpnHzlilphb1IP1xqEHBYoFHMczaGYSS8wySXlKPRPs4VjbFsrWGs42VctKTLhMI5FsFa8njMQm1Ky18Abu9OAGzs39bBmDKcC15K3dC8a8M5O%2FANKBZImy7UDzO2kE4dOedXhoGXMQyvKg4EJvrcos28JgYgxKZ8fOQOGoTbOArw7GzjotnRneaR9FDwZCEK4oruk1sjtpQcfjgEEBZL42aKy6Aea%2BhTv4uBAP4N9SJesEmzU6y6d9lIbuiOlBgA74hP6BgoyCcR8uciUu%2FT5eswehBnhVpdIEbI%2BALACjSdOt3HBD1kM8mjbh8f1MzArX%2B%2B3ubG6nfV%2FdS8Y%2BwRnSHo3%2FY0UeopLbeWT1stmb7YUfGimtLF78%2BuXDUyOlPC2%2FU1FNh76uCpo4eXtKvjfvSsvLsXStifRnMZz6dP0HetcFbhcHAAA%3D)).

> Why convert IIFEs instead of enums themselves? See [this issue](https://github.com/swc-project/swc/issues/7501).

## Examples

```ts
// Before
enum SimpleNumber {
  Bar,
  Baz,
}

// After
var SimpleNumber = {
  "Bar": 0,
  0: "Bar",
  "Baz": 1,
  1: "Baz"
};
```

```ts
// Before
export enum SimpleString {
  A = 'Z',
  B = 'Y',
}

// After
export var SimpleString = {
  "A": 'Z',
  "B": 'Y'
};
```

## Caveats

- **Enum merging might not work.**

  This plugin strictly looks for the "`var X` + IIFE" pattern.
  Mergings will be left unchanged.

  ```ts
  // Before
  enum Foo { Bar = 1 }
  enum Foo { Baz = 2 }

  // After
  var Foo = {
      "Bar": 1,
      1: "Bar"
  };
  (function(Foo) {
      Foo[Foo["Baz"] = 2] = "Baz";
  })(Foo || (Foo = {}));
  ```

- **You might accidently convert enums of external libraries.**

  It is a problem caused by converting IIFEs.

  If you are using libraries containing enums, and you make SWC process all node_modules file at the same time,
  those enums will be converted too because they also match the "`var X` + IIFE" pattern.

  This should not cause any problems. However I think you should notice this behavior.

- **Enums containing complex caclucations are not supported.**

  This plugin strictly looks for number or string literals.
  However in some cases, SWC left calculation expressions stay in the IIFE. This kind of enums will be left unchanged.

  ```ts
  // Before
  enum Foo {
    A = 3,
    B = 2,
    C = 1,
    X = A ** B ** C,
  }

  // After
  var Foo;
  (function(Foo) {
      Foo[Foo["A"] = 3] = "A";
      Foo[Foo["B"] = 2] = "B";
      Foo[Foo["C"] = 1] = "C";
      Foo[Foo["X"] = Foo.A ** Foo.B ** Foo.C] = "X";
  })(Foo || (Foo = {}));
  ```

- **`const enum`s are handled by SWC, not this plugin.**

  SWC currently converts `const enum`s the same way as normal enums. There will be an enum object as a result.

## License

MIT
