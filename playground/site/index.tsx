import { ferric, init } from "../pkg";
import React, { useState } from "react";
import { useRef } from "react";
import { createRoot } from "react-dom/client";

import Editor from "@monaco-editor/react";
import { useEffect } from "react";

import "./style.css";

const INITIAL_TEXT = `print("Hello, World!");`;

const Main = () => {
  const [text, setText] = useState(INITIAL_TEXT);
  const output = useRef<HTMLPreElement>(null);
  
  useEffect(() => {
    console.log(output.current);
    if (output.current != null) {
      init(output.current);
    }
  }, [output]);

  return (
    <div className="min-h-screen bg-slate-800 px-6 py-4">
      <button
        className="m-4 rounded border bg-slate-300 px-2 py-1"
        onClick={() => {
          if (output.current === null) {
            alert("Couldn't find output");
          } else {
            ferric(text, output.current);
          }
        }}
      >
        Run
      </button>
      <div className="grid grid-cols-2 gap-4">
        <div className="border">
          <Editor
            defaultValue={INITIAL_TEXT}
            theme="vs-dark"
            height="90vh"
            onChange={(value, ev) => {
              setText(value ?? "undefined");
            }}
            options={{
              minimap: {
                enabled: false,
              },
            }}
          />
        </div>
        <div className="flex h-[90vh] flex-col gap-4">
          <pre
            className="grow overflow-auto border border-black bg-slate-700 p-2 text-white"
            ref={output}
          ></pre>
          {/*<div className="border bg-slate-700 p-2 text-white border-black grow overflow-auto">
            <p class="text-base mb-4">
              Ferric is an expression based language, so each expression must
              evaluate to a value. Here's a list of every implemented
              expression.
            </p>
            <ul class="list-disc pl-6 mb-4">
              <li class="mb-2">
                Literal - A literal value. Can be a numeric literal (
                <code class="bg-gray-800 text-white p-2 rounded-md">4.5</code>),
                or a string literal (
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  &quot;Hello, World!&quot;
                </code>
                ).
              </li>
              <li class="mb-2">
                Identifier - The name of something. It could be a variable or a
                built-in function.
              </li>
              <li class="mb-2">
                Binary - A binary expression. The valid operators are:{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">+</code>,{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">-</code>,{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">*</code>,{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">/</code>,{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">=</code>,{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">!=</code>,{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">&gt;</code>,{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">&gt;</code>,{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">&gt;=</code>
                ,{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">&lt;=</code>
                . Ex:{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">4 + 5</code>
              </li>
              <li class="mb-2">
                Unary - A unary expression. The valid operators are:{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">~</code>{" "}
                (bitwise not) and{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">-</code>{" "}
                (negate). Ex:{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  -(1 + 2)
                </code>
              </li>
              <li class="mb-2">
                Call - A function call. See the built-in functions below. Ex:{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  print(&quot;Hello, World!&quot;)
                </code>
              </li>
              <li class="mb-2">
                Declaration - A variable declaration. Begins with{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">let</code>,
                followed by the name of the variable, then an equal sign,
                followed by its initial value. Declarations evaluate to{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">Null</code>.
                Ex:{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  let x = 4
                </code>
              </li>
              <li class="mb-2">
                Assignment - Change a variables value. Begins with the
                variable's name, followed by an equal sign, then the new value
                of the variable. Assignments evaluate to Null. Ex:{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">x = 4</code>
              </li>
              <li class="mb-2">
                Block - A list of expressions separated by semicolons. If the
                last expression is followed by a semicolon, the block evaluates
                to Null. If the last expression is not followed by a semicolon,
                the block evaluates to the last expression. The values of
                expressions followed by semicolons are discarded. Ex:{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  &lbrace;print(&quot;hi&quot;); 1 + 2&rcub;
                </code>{" "}
                evaluates to{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">3</code>.
              </li>
              <li class="mb-2">
                If - An if expression. If expressions begin with the{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">if</code>{" "}
                keyword, followed by the condition. The condition must evaluate
                to a boolean. Then comes the &quot;then&quot; block, followed by
                an optional &quot;otherwise&quot; block preceded by the{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  otherwise
                </code>{" "}
                keyword. The if expression evaluates to the value of the
                selected block. Ex:{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  if 1 == 1 &lbrace;&quot;that's true&quot;&rbrace; otherwise
                  &lbrace;&quot;that's false&quot;&rbrace;
                </code>{" "}
                evaluates to{" "}
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  &quot;that's true&quot;
                </code>
                .
              </li>
              <li class="mb-2">
                While - A while loop. While loops begin with the while keyword,
                followed by a condition, then a block containing the loop's
                body. While loops evaluate to Null.
              </li>
            </ul>
            <h2 class="text-3xl font-semibold mb-3">Built-in functions</h2>
            <ul class="list-disc pl-6 mb-4">
              <li class="mb-2">
                <code class="bg-gray-800 text-white p-2 rounded-md">print</code>
              </li>
            </ul>
            <pre class="bg-gray-100 p-4 rounded-md overflow-x-auto">
              <code class="language-ts">
                // Prints a serialized version of the input to the output
                variable stored in the interpreter. print(...args: any)
              </code>
            </pre>
            <ul class="list-disc pl-6 mb-4">
              <li class="mb-2">
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  fs_read
                </code>
              </li>
            </ul>
            <pre class="bg-gray-100 p-4 rounded-md overflow-x-auto">
              <code class="language-ts">
                // Reads from a file on disk and returns its contents as a
                string. Panics if the file doesn’t exist. fs_read(file: string)
                -&gt; string
              </code>
            </pre>
            <ul class="list-disc pl-6 mb-4">
              <li class="mb-2">
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  fs_write
                </code>
              </li>
            </ul>
            <pre class="bg-gray-100 p-4 rounded-md overflow-x-auto">
              <code class="language-ts">
                // Overwrites a file on disk with new contents. If the file
                doesn’t exist, create a new file. fs_write(file: string,
                contents: string)
              </code>
            </pre>
            <ul class="list-disc pl-6 mb-4">
              <li class="mb-2">
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  unix_time
                </code>
              </li>
            </ul>
            <pre class="bg-gray-100 p-4 rounded-md overflow-x-auto">
              <code class="language-ts">
                // Returns the number of seconds since Jan 1st, 1970 (UNIX
                Epoch) as a floating point number. unix_time() -&gt; number
              </code>
            </pre>
            <ul class="list-disc pl-6 mb-4">
              <li class="mb-2">
                <code class="bg-gray-800 text-white p-2 rounded-md">clock</code>
              </li>
            </ul>
            <pre class="bg-gray-100 p-4 rounded-md overflow-x-auto">
              <code class="language-ts">
                // Returns the number of seconds since the interpreter was
                started. clock() -&gt; number
              </code>
            </pre>
            <ul class="list-disc pl-6 mb-4">
              <li class="mb-2">
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  substring
                </code>
              </li>
            </ul>
            <pre class="bg-gray-100 p-4 rounded-md overflow-x-auto">
              <code class="language-ts">
                // Returns a specific substring of a string given the starting
                and ending byte of the substring. Panics if start or end is not
                an integer. substring(s: string, start: number, end: number)
                -&gt; string
              </code>
            </pre>
            <ul class="list-disc pl-6 mb-4">
              <li class="mb-2">
                <code class="bg-gray-800 text-white p-2 rounded-md">
                  strlen
                </code>
              </li>
            </ul>
            <pre class="bg-gray-100 p-4 rounded-md overflow-x-auto">
            </pre>
          </div> */}
        </div>
      </div>
    </div>
  );
};

/*
## Ferric Documentation

[Ferric GitHub](https://github.com/david-callender/Ferric)

Ferric is a reference-counted, tree-walking scripting language implemented in Rust. Instead of using a garbage collector to manage memory, Ferric keeps track of how many times a variable is referenced and deletes the variable when it is no longer referenced. As opposed to other languages that compile to bytecode, Ferric directly interprets the expressions produced by the parser. Both of these decisions make development simpler while only compromising minimal speed.

The Ferric runtime is separated into 3 parts:
* The Lexer - It is the Lexer's job to take the incoming stream of bytes and figure out which ones should be grouped together. These groups are called `Tokens`. These tokens can be a keyword, an identifier, a number literal, etc. The Lexer also strips out any whitespace and comments.
* The Parser - After recieving the stream of tokens output by the Lexer, the Parser figures out in which order they need to be evaluated in. This order is enforced using an Abstract Syntax Tree, or AST. Each node in the tree might contain other AST nodes, allowing for expressions to contain other expressions.

Ferric is an *expression based language*, similar to Haskell or OCaml, where everything must evaluate to an expression.
*/

// const app = document.createElement("div");
// app.id = "app";
// document.body.append(app);

const body = document.body;
const root = createRoot(body);

root.render(<Main />);
