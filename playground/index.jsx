import { ferric, init } from "./pkg";
import React, { useState } from "react";
import { useRef } from "react";
import { createRoot } from "react-dom/client";

import Editor from "@monaco-editor/react";
import { useEffect } from "react";

//@ts-check

const INITIAL_TEXT = `print("Hello, World!");`;

const Component = () => {
  const [text, setText] = useState(INITIAL_TEXT);
  const output = useRef(null);

  useEffect(() => {
    console.log(output.current);
    if (output != null) {
      init(output.current);
    }
  }, [output]);

  return (
    <div className="px-6 py-4 min-h-screen bg-slate-800">
      <button
        className="bg-slate-300 border rounded px-2 py-1 m-4"
        onClick={() => ferric(text, output.current)}
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
              setText(value);
            }}
            options={{
              minimap: {
                enabled: false,
              },
            }}
          />
        </div>
        <pre
          className="border bg-slate-700 p-2 text-white border-black h-[90vh] overflow-auto"
          ref={output}
        ></pre>
      </div>
    </div>
  );
};

const app = document.createElement("div");
app.id = "app";
document.body.append(app);

const root = createRoot(app);

root.render(<Component />);
