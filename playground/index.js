import { ferric } from "./pkg";

const textArea = document.createElement("textarea");
document.body.append(textArea);

const run = document.createElement("button");
document.body.append(run);
run.textContent = "Run";
run.onclick = () => {
    ferric(textArea.value);
};

const output = document.createElement("pre");
output.id = "output";
document.body.append(output);
