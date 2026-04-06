import { Editor } from "@monaco-editor/react";
import React, { useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import Markdown from "react-markdown";

import { ferric, init } from "../pkg";

import infoMarkdown from "./info.md";
import "./style.css";

import type { editor } from "monaco-editor";
import type { FC } from "react";

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
    if (output.current !== null) {
      init(output.current);
    }
  }, [output]);

  const handleRun = (): void => {
    if (output.current === null) {
      alert("Couldn't find output");
    } else {
      ferric(text, output.current);
    }
  };

  const handleChange = (
    value: string | undefined,
    _: editor.IModelContentChangedEvent
  ): void => {
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

const root = createRoot(document.body);
root.render(<Main />);
