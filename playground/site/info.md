## Ferric Usage

Ferric is an expression based language, so each expression must evaluate to a value. Here's a list of every implemented expression.

* Literal - A literal value. Can be a numeric literal (`4.5`), or a string literal (`"Hello, World!"`).
* Identifier - The name of something. It could be a variable or a built-in function.
* Binary - A binary expression. The valid operators are: `+`, `-`, `*`, `/`, `=`, `!=`, `>`, `>`, `>=`, `<=`. Ex: `4 + 5`
* Unary - A unary expression. The valid operators are: `~` (bitwise not) and `-` (negate). Ex: `-(1 + 2)`
* Call - A function call. See the built-in functions below. Ex: `print("Hello, World!")`
* Declaration - A variable declaration. Begins with `let`, followed by the name of the variable, then an equal sign, followed by its initial value. Declarations evaluate to `Null`. Ex: `let x = 4`
* Assignment - Change a variables value. Begins with the variable's name, followed by an equal sign, then the new value of the variable. Assignments evaluate to Null. Ex: `x = 4`
* Block - A list of expressions separated by semicolons. If the last expression is followed by a semicolon, the block evaluates to Null. If the last expression is not followed by a semicolon, the block evaluates to the last expression. The values of expressions followed by semicolons are discarded. Ex: `{print("hi"); 1 + 2}` evaluates to `3`.
* If - An if expression. If expressions begin with the `if` keyword, followed by the condition. The condition must evaluate to a boolean. Then comes the "then" block, followed by an optional "otherwise" block preceded by the `otherwise` keyword. The if expression evaluates to the value of the selected block. Ex: `if 1 == 1 {"that's true"} otherwise {"that's false"}` evaluates to `"that's true"`.
* While - A while loop. While loops begin with the while keyword, followed by a condition, then a block containing the loop's body. While loops evaluate to Null.

## Built-in functions

 - `print`
```ts
// Prints a serialized version of the input to the output variable stored in the interpreter.
print(...args: any)
```

- `fs_read`
```ts
// Reads from a file on disk and returns its contents as a string. Panics if the file doesn’t exist.
fs_read(file: string) -> string
```

- `fs_write`
```ts
// Overwrites a file on disk with new contents. If the file doesn’t exist, create a new file.
fs_write(file: string, contents: string)
```

- `unix_time`
```ts
// Returns the number of seconds since Jan 1st, 1970 (UNIX Epoch) as a floating point number.
unix_time() -> number
```

- `clock`
```ts
// Returns the number of seconds since the interpreter was started.
clock() -> number
```

- `substring`
```ts
// Returns a specific substring of a string given the starting and ending byte of the substring. Panics if start or end is not an integer.
substring(s: string, start: number, end: number) -> string
```

- `strlen`
```ts
// Returns the length of a string in bytes.
strlen(s: string) -> number
```