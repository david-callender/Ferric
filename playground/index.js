const textArea = document.createElement("textarea");
document.body.append(textArea);

const run = document.createElement("button");
document.body.append(run);

const output = document.createElement("pre");
output.id = "output";
document.body.append(output);


import { ferric } from "./pkg";

run.textContent = "Run";
run.onclick = () => {
    ferric(textArea.value);
};



// ferric(`
//   let a = 4;
//   print(a);
//   if a == 4 {
//     print("its 4!");
//   } otherwise {
//     print("its not 4!");
//   };
// `);
