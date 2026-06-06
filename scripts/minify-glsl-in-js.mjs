import { readFile, writeFile } from "node:fs/promises";

const SHADER_KEYS = ["vertexShader", "fragmentShader"];

function stripLineComment(line) {
  let quote = null;

  for (let i = 0; i < line.length - 1; i++) {
    const char = line[i];
    const next = line[i + 1];

    if (quote) {
      if (char === "\\") {
        i++;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === '"' || char === "'") {
      quote = char;
      continue;
    }

    if (char === "/" && next === "/") {
      return line.slice(0, i);
    }
  }

  return line;
}

function compactCode(code) {
  return code
    .replace(/\s+/g, " ")
    .replace(/\s*([{}()[\],;:?])\s*/g, "$1")
    .replace(/\s*([*/%<>=!+\-])\s*/g, "$1")
    .trim();
}

function minifyGlsl(source) {
  const withoutBlockComments = source.replace(/\/\*[\s\S]*?\*\//g, "");
  const output = [];
  let code = "";

  for (const rawLine of withoutBlockComments.split(/\r?\n/)) {
    const line = stripLineComment(rawLine).trim();
    if (!line) {
      continue;
    }

    if (line.startsWith("#")) {
      const compacted = compactCode(code);
      if (compacted) {
        output.push(compacted);
        code = "";
      }
      output.push(line.replace(/\s+/g, " "));
      continue;
    }

    code += `${line} `;
  }

  const compacted = compactCode(code);
  if (compacted) {
    output.push(compacted);
  }

  return output.join("\n");
}

function parseQuotedLiteral(source, start) {
  const quote = source[start];
  let i = start + 1;

  while (i < source.length) {
    const char = source[i];

    if (char === "\\") {
      i += 2;
      continue;
    }

    if (char === quote) {
      const literal = source.slice(start, i + 1);
      return {
        end: i + 1,
        replacement: JSON.stringify(minifyGlsl(Function(`return ${literal}`)())),
      };
    }

    i++;
  }

  return null;
}

function parseTemplateExpression(source, start) {
  let depth = 1;
  let i = start;
  let quote = null;
  let lineComment = false;
  let blockComment = false;
  let templateDepth = 0;

  while (i < source.length) {
    const char = source[i];
    const next = source[i + 1];

    if (lineComment) {
      if (char === "\n" || char === "\r") {
        lineComment = false;
      }
      i++;
      continue;
    }

    if (blockComment) {
      if (char === "*" && next === "/") {
        blockComment = false;
        i += 2;
      } else {
        i++;
      }
      continue;
    }

    if (quote) {
      if (char === "\\") {
        i += 2;
      } else if (char === quote) {
        quote = null;
        i++;
      } else {
        i++;
      }
      continue;
    }

    if (templateDepth > 0) {
      if (char === "\\") {
        i += 2;
      } else if (char === "`") {
        templateDepth--;
        i++;
      } else if (char === "$" && next === "{") {
        depth++;
        i += 2;
      } else {
        i++;
      }
      continue;
    }

    if (char === "/" && next === "/") {
      lineComment = true;
      i += 2;
      continue;
    }

    if (char === "/" && next === "*") {
      blockComment = true;
      i += 2;
      continue;
    }

    if (char === '"' || char === "'") {
      quote = char;
      i++;
      continue;
    }

    if (char === "`") {
      templateDepth++;
      i++;
      continue;
    }

    if (char === "{") {
      depth++;
    } else if (char === "}") {
      depth--;
      if (depth === 0) {
        return i;
      }
    }

    i++;
  }

  return -1;
}

function escapeTemplateQuasi(value) {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/`/g, "\\`")
    .replace(/\$\{/g, "\\${");
}

function parseTemplateLiteral(source, start) {
  let i = start + 1;
  let quasi = "";
  let replacement = "`";

  while (i < source.length) {
    const char = source[i];
    const next = source[i + 1];

    if (char === "\\") {
      quasi += char + next;
      i += 2;
      continue;
    }

    if (char === "`") {
      replacement += escapeTemplateQuasi(minifyGlsl(quasi));
      replacement += "`";
      return { end: i + 1, replacement };
    }

    if (char === "$" && next === "{") {
      replacement += escapeTemplateQuasi(minifyGlsl(quasi));
      const expressionEnd = parseTemplateExpression(source, i + 2);
      if (expressionEnd === -1) {
        return null;
      }
      replacement += ` ${source.slice(i, expressionEnd + 1)} `;
      quasi = "";
      i = expressionEnd + 1;
      continue;
    }

    quasi += char;
    i++;
  }

  return null;
}

function findShaderLiteralStart(source, keyStart, key) {
  let i = keyStart + key.length;

  while (/\s/.test(source[i] ?? "")) {
    i++;
  }

  if (source[i] !== ":" && source[i] !== "=") {
    return -1;
  }

  i++;

  while (/\s/.test(source[i] ?? "")) {
    i++;
  }

  return source[i] === '"' || source[i] === "'" || source[i] === "`" ? i : -1;
}

function minifyShaderLiterals(source) {
  const replacements = [];

  for (const key of SHADER_KEYS) {
    let searchStart = 0;

    while (searchStart < source.length) {
      const keyStart = source.indexOf(key, searchStart);
      if (keyStart === -1) {
        break;
      }

      const literalStart = findShaderLiteralStart(source, keyStart, key);
      if (literalStart === -1) {
        searchStart = keyStart + key.length;
        continue;
      }

      const parsed =
        source[literalStart] === "`"
          ? parseTemplateLiteral(source, literalStart)
          : parseQuotedLiteral(source, literalStart);

      if (parsed) {
        replacements.push({
          start: literalStart,
          end: parsed.end,
          replacement: parsed.replacement,
        });
        searchStart = parsed.end;
      } else {
        searchStart = literalStart + 1;
      }
    }
  }

  replacements.sort((a, b) => b.start - a.start);

  let output = source;
  for (const { start, end, replacement } of replacements) {
    output = output.slice(0, start) + replacement + output.slice(end);
  }

  return { output, count: replacements.length };
}

for (const file of process.argv.slice(2)) {
  const source = await readFile(file, "utf8");
  const { output, count } = minifyShaderLiterals(source);

  if (output !== source) {
    await writeFile(file, output);
  }

  console.log(`Minified ${count} GLSL shader literal${count === 1 ? "" : "s"} in ${file}`);
}
