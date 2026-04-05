declare module "*.css";
declare module "*.md" {
  const contents: string;
  export = contents;
}
