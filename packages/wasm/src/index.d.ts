export interface FormatOptions {
  indentWidth?: number;
  lineWidth?: number;
  maxBlankLines?: number;
}
export declare function formatText(fileText: string, options?: FormatOptions): string;
