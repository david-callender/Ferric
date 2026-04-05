import { ferric, init } from "../pkg";
import React, { FC, useState } from "react";
import { useRef } from "react";
import { createRoot } from "react-dom/client";

import Editor from "@monaco-editor/react";
import { useEffect } from "react";

import "./style.css";
import { editor } from "monaco-editor";

import Markdown from "react-markdown";

import infoMarkdown from "./info.md";

const INITIAL_TEXT = `print("Hello, World!");`;

const FerricInfo: FC = () => {
  return (
    <article className="prose prose-invert prose-code:before:content-none prose-code:after:content-none mx-auto max-w-4xl text-lg">
      <Markdown>{infoMarkdown}</Markdown>
    </article>
  );
};

const Main: FC = () => {
  const [text, setText] = useState(INITIAL_TEXT);
  const output = useRef<HTMLPreElement>(null);

  useEffect(() => {
    if (output.current != null) {
      init(output.current);
    }
  }, [output]);

  const handleRun = () => {
    if (output.current === null) {
      alert("Couldn't find output");
    } else {
      ferric(text, output.current);
    }
  };

  const handleChange = (
    value: string | undefined,
    _: editor.IModelContentChangedEvent
  ) => {
    setText(value ?? "undefined");
  };

  return (
    <div className="min-h-screen bg-slate-900 p-6 text-white">
      <div className="text-center text-3xl font-bold">Ferric Playground</div>
      <button
        className="mb-2 rounded border border-black bg-slate-700 px-2 py-1 hover:cursor-pointer"
        onClick={handleRun}
      >
        Run
      </button>
      <div className="mb-8 grid grid-cols-2 gap-4">
        <div className="border border-black">
          <Editor
            defaultValue={INITIAL_TEXT}
            theme="vs-dark"
            height="70vh"
            onChange={handleChange}
            options={{
              minimap: {
                enabled: false,
              },
            }}
          />
        </div>
        <div className="flex h-[70vh] flex-col gap-4">
          <pre
            className="grow overflow-auto border border-black bg-slate-800 p-2"
            ref={output}
          >
            Output will appear here
          </pre>
        </div>
      </div>
      <FerricInfo />
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
