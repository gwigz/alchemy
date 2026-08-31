import { Resvg } from "@resvg/resvg-js"
import {
  copyFile,
  mkdir,
  readFile,
  readdir,
  unlink,
  writeFile,
} from "node:fs/promises"
import { inflateSync } from "node:zlib"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"

const CANVAS_SIZE = 16
const LUCIDE_VIEWBOX_SIZE = 24
const DEFAULT_STROKE_WIDTH = 1.75
const MAX_ICON_OFFSET = 2
const PNG_SIGNATURE = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])
const XML_BEGIN = "  <!-- BEGIN Inventory Explorer generated textures -->"
const XML_END = "  <!-- END Inventory Explorer generated textures -->"

interface IconSpec {
  texture: string
  lucide: string
  size: number
  strokeWidth: number
  offsetX: number
  offsetY: number
}

const toolRoot = dirname(fileURLToPath(import.meta.url))
const alchemyRoot = resolve(toolRoot, "../..")
const lucideRoot = resolve(alchemyRoot, "../viewer/fonts/lucide")
const outputRoot = resolve(
  alchemyRoot,
  "indra/newview/skins/default/textures/inventory_explorer"
)
const texturesXmlPath = resolve(
  alchemyRoot,
  "indra/newview/skins/default/textures/textures.xml"
)
const manifestPath = resolve(toolRoot, "manifest.json")
const sourceLicensePath = resolve(lucideRoot, "LICENSE")
const licensePath = resolve(toolRoot, "LUCIDE-LICENSE.txt")
const contactSheetPath = resolve(toolRoot, "contact-sheet.png")
const checkOnly = process.argv.includes("--check")

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}

function readManifest(value: unknown): IconSpec[] {
  if (!Array.isArray(value)) {
    throw new Error("manifest.json must contain an array")
  }

  const entries = value.map((entry, index) => {
    if (!isRecord(entry)) {
      throw new Error(`manifest entry ${index + 1} must be an object`)
    }
    const { texture, lucide, size } = entry
    const strokeWidth = "strokeWidth" in entry
      ? entry.strokeWidth
      : DEFAULT_STROKE_WIDTH
    const offsetX = "offsetX" in entry ? entry.offsetX : 0
    const offsetY = "offsetY" in entry ? entry.offsetY : 0
    if (
      typeof texture !== "string" ||
      !/^InvExplorer_[A-Za-z0-9_]+$/.test(texture)
    ) {
      throw new Error(`manifest entry ${index + 1} has an invalid texture name`)
    }
    if (typeof lucide !== "string" || !/^[a-z0-9-]+$/.test(lucide)) {
      throw new Error(`manifest entry ${index + 1} has an invalid Lucide name`)
    }
    if (
      typeof size !== "number" ||
      !Number.isInteger(size) ||
      size < 1 ||
      size > CANVAS_SIZE
    ) {
      throw new Error(`manifest entry ${index + 1} has an invalid size`)
    }
    if (
      typeof strokeWidth !== "number" ||
      !Number.isFinite(strokeWidth) ||
      strokeWidth < 1 ||
      strokeWidth > 3 ||
      !Number.isInteger(strokeWidth * 4)
    ) {
      throw new Error(
        `manifest entry ${index + 1} has an invalid stroke width`
      )
    }
    if (
      typeof offsetX !== "number" ||
      !Number.isFinite(offsetX) ||
      Math.abs(offsetX) > MAX_ICON_OFFSET ||
      !Number.isInteger(offsetX * 4)
    ) {
      throw new Error(`manifest entry ${index + 1} has an invalid x offset`)
    }
    if (
      typeof offsetY !== "number" ||
      !Number.isFinite(offsetY) ||
      Math.abs(offsetY) > MAX_ICON_OFFSET ||
      !Number.isInteger(offsetY * 4)
    ) {
      throw new Error(`manifest entry ${index + 1} has an invalid y offset`)
    }
    return { texture, lucide, size, strokeWidth, offsetX, offsetY }
  })

  const names = new Set<string>()
  for (const entry of entries) {
    if (names.has(entry.texture)) {
      throw new Error(`duplicate texture name: ${entry.texture}`)
    }
    names.add(entry.texture)
  }

  return entries
}

function escapeXml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
}

function extractSvgBody(source: string, sourceName: string): string {
  const body = source.match(/<svg\b[^>]*>([\s\S]*?)<\/svg>\s*$/)?.[1]
  if (body === undefined) {
    throw new Error(`cannot read SVG body: ${sourceName}`)
  }
  return body.replaceAll("currentColor", "#ffffff")
}

function renderIcon(source: string, spec: IconSpec): Buffer {
  const x = (CANVAS_SIZE - spec.size) / 2 + spec.offsetX
  const y = (CANVAS_SIZE - spec.size) / 2 + spec.offsetY
  const sourceStrokeWidth =
    (spec.strokeWidth * LUCIDE_VIEWBOX_SIZE) / spec.size
  const svg = [
    `<svg xmlns="http://www.w3.org/2000/svg" width="${CANVAS_SIZE}" height="${CANVAS_SIZE}" viewBox="0 0 ${CANVAS_SIZE} ${CANVAS_SIZE}">`,
    `<svg x="${x}" y="${y}" width="${spec.size}" height="${spec.size}" viewBox="0 0 ${LUCIDE_VIEWBOX_SIZE} ${LUCIDE_VIEWBOX_SIZE}" preserveAspectRatio="xMidYMid meet" fill="none" stroke="#ffffff" stroke-width="${sourceStrokeWidth}" stroke-linecap="round" stroke-linejoin="round" shape-rendering="geometricPrecision">`,
    extractSvgBody(source, spec.lucide),
    "</svg>",
    "</svg>",
  ].join("")
  return Buffer.from(
    new Resvg(svg, {
      fitTo: { mode: "width", value: CANVAS_SIZE },
    })
      .render()
      .asPng()
  )
}

function paeth(a: number, b: number, c: number): number {
  const p = a + b - c
  const pa = Math.abs(p - a)
  const pb = Math.abs(p - b)
  const pc = Math.abs(p - c)
  if (pa <= pb && pa <= pc) return a
  return pb <= pc ? b : c
}

function validatePng(png: Buffer, texture: string): void {
  if (!png.subarray(0, PNG_SIGNATURE.length).equals(PNG_SIGNATURE)) {
    throw new Error(`${texture}: invalid PNG signature`)
  }

  let offset = PNG_SIGNATURE.length
  let width = 0
  let height = 0
  let bitDepth = 0
  let colorType = 0
  const idat: Buffer[] = []
  while (offset < png.length) {
    const length = png.readUInt32BE(offset)
    const type = png.toString("ascii", offset + 4, offset + 8)
    const data = png.subarray(offset + 8, offset + 8 + length)
    if (type === "IHDR") {
      width = data.readUInt32BE(0)
      height = data.readUInt32BE(4)
      bitDepth = data[8]
      colorType = data[9]
      if (data[10] !== 0 || data[11] !== 0 || data[12] !== 0) {
        throw new Error(`${texture}: unsupported PNG encoding`)
      }
    } else if (type === "IDAT") {
      idat.push(data)
    } else if (type === "IEND") {
      break
    }
    offset += length + 12
  }

  if (width !== CANVAS_SIZE || height !== CANVAS_SIZE) {
    throw new Error(`${texture}: expected 16 x 16, got ${width} x ${height}`)
  }
  if (bitDepth !== 8 || colorType !== 6) {
    throw new Error(`${texture}: expected an 8-bit RGBA PNG`)
  }

  const bytesPerPixel = 4
  const rowLength = width * bytesPerPixel
  const inflated = inflateSync(Buffer.concat(idat))
  if (inflated.length !== height * (rowLength + 1)) {
    throw new Error(`${texture}: unexpected decompressed PNG length`)
  }

  let previous = Buffer.alloc(rowLength)
  let alphaMin = 255
  let alphaMax = 0
  for (let y = 0; y < height; y += 1) {
    const rowStart = y * (rowLength + 1)
    const filter = inflated[rowStart]
    const encoded = inflated.subarray(rowStart + 1, rowStart + 1 + rowLength)
    const row = Buffer.alloc(rowLength)
    for (let x = 0; x < rowLength; x += 1) {
      const left = x >= bytesPerPixel ? row[x - bytesPerPixel] : 0
      const above = previous[x]
      const upperLeft = x >= bytesPerPixel ? previous[x - bytesPerPixel] : 0
      let predictor = 0
      switch (filter) {
        case 0:
          break
        case 1:
          predictor = left
          break
        case 2:
          predictor = above
          break
        case 3:
          predictor = Math.floor((left + above) / 2)
          break
        case 4:
          predictor = paeth(left, above, upperLeft)
          break
        default:
          throw new Error(`${texture}: unsupported PNG row filter ${filter}`)
      }
      row[x] = (encoded[x] + predictor) & 0xff
    }
    for (let x = 3; x < rowLength; x += bytesPerPixel) {
      alphaMin = Math.min(alphaMin, row[x])
      alphaMax = Math.max(alphaMax, row[x])
    }
    previous = row
  }

  if (alphaMin !== 0 || alphaMax === 0) {
    throw new Error(`${texture}: expected transparent and visible pixels`)
  }
}

function textureBlock(specs: IconSpec[]): string {
  const entries = [...specs]
    .sort((a, b) => a.texture.localeCompare(b.texture))
    .map(
      ({ texture }) =>
        `  <texture name="${texture}" file_name="inventory_explorer/${texture}.png" preload="false" />`
    )
  return [XML_BEGIN, ...entries, XML_END].join("\n")
}

function updateTexturesXml(xml: string, block: string, specs: IconSpec[]): string {
  const begin = xml.indexOf(XML_BEGIN)
  const end = xml.indexOf(XML_END)
  let withoutGenerated = xml
  if ((begin === -1) !== (end === -1) || (begin !== -1 && end < begin)) {
    throw new Error("textures.xml has malformed Inventory Explorer markers")
  }
  if (begin !== -1) {
    withoutGenerated =
      xml.slice(0, begin) + xml.slice(end + XML_END.length)
  }

  for (const { texture } of specs) {
    const duplicate = new RegExp(`name=["']${texture}["']`).test(
      withoutGenerated
    )
    if (duplicate) {
      throw new Error(`textures.xml already declares ${texture} outside the generated block`)
    }
  }

  if (begin !== -1) {
    return xml.slice(0, begin) + block + xml.slice(end + XML_END.length)
  }
  const closing = xml.lastIndexOf("</textures>")
  if (closing === -1) {
    throw new Error("textures.xml has no closing textures element")
  }
  return `${xml.slice(0, closing).trimEnd()}\n\n${block}\n</textures>\n`
}

function renderContactSheet(
  specs: IconSpec[],
  icons: ReadonlyMap<string, Buffer>
): Buffer {
  const columns = 4
  const cellWidth = 220
  const cellHeight = 92
  const rows = Math.ceil(specs.length / columns)
  const width = columns * cellWidth
  const height = rows * cellHeight
  const cells = specs.map((spec, index) => {
    const column = index % columns
    const row = Math.floor(index / columns)
    const x = column * cellWidth
    const y = row * cellHeight
    const png = icons.get(spec.texture)
    if (png === undefined) {
      throw new Error(`missing rendered icon for ${spec.texture}`)
    }
    const href = `data:image/png;base64,${png.toString("base64")}`
    return [
      `<rect x="${x + 1}" y="${y + 1}" width="${cellWidth - 2}" height="${cellHeight - 2}" rx="5" fill="#17191d" stroke="#343941"/>`,
      `<rect x="${x + 11}" y="${y + 11}" width="18" height="18" fill="#050608" stroke="#343941"/>`,
      `<image x="${x + 12}" y="${y + 12}" width="16" height="16" href="${href}"/>`,
      `<image x="${x + 42}" y="${y + 8}" width="64" height="64" href="${href}" style="image-rendering:pixelated"/>`,
      `<text x="${x + 114}" y="${y + 30}" fill="#f2f4f8" font-family="monospace" font-size="11">${escapeXml(spec.texture)}</text>`,
      `<text x="${x + 114}" y="${y + 49}" fill="#9198a4" font-family="monospace" font-size="10">${escapeXml(spec.lucide)} · ${spec.size}px · ${spec.strokeWidth}px stroke</text>`,
      `<text x="${x + 12}" y="${y + 83}" fill="#717784" font-family="sans-serif" font-size="9">native 16 x 16</text>`,
    ].join("")
  })
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}"><rect width="100%" height="100%" fill="#090a0c"/>${cells.join("")}</svg>`
  return Buffer.from(new Resvg(svg).render().asPng())
}

async function writeOrCheck(path: string, expected: Buffer | string): Promise<void> {
  const expectedBuffer = Buffer.isBuffer(expected)
    ? expected
    : Buffer.from(expected)
  let actual: Buffer | undefined
  try {
    actual = await readFile(path)
  } catch (error) {
    if (!isRecord(error) || error.code !== "ENOENT") throw error
  }

  if (checkOnly) {
    if (actual === undefined || !actual.equals(expectedBuffer)) {
      throw new Error(`generated output differs: ${path}`)
    }
    return
  }
  if (actual === undefined || !actual.equals(expectedBuffer)) {
    await mkdir(dirname(path), { recursive: true })
    await writeFile(path, expectedBuffer)
    console.log(`wrote ${path}`)
  }
}

const manifestValue: unknown = JSON.parse(await readFile(manifestPath, "utf8"))
const specs = readManifest(manifestValue)
const icons = new Map<string, Buffer>()
for (const spec of specs) {
  const sourcePath = resolve(lucideRoot, "icons", `${spec.lucide}.svg`)
  let source: string
  try {
    source = await readFile(sourcePath, "utf8")
  } catch (error) {
    if (isRecord(error) && error.code === "ENOENT") {
      throw new Error(`missing Lucide source icon: ${sourcePath}`)
    }
    throw error
  }
  const png = renderIcon(source, spec)
  validatePng(png, spec.texture)
  icons.set(spec.texture, png)
}

if (!checkOnly) {
  await mkdir(outputRoot, { recursive: true })
  const expectedFiles = new Set(specs.map(({ texture }) => `${texture}.png`))
  for (const file of await readdir(outputRoot)) {
    if (file.endsWith(".png") && !expectedFiles.has(file)) {
      const stalePath = resolve(outputRoot, file)
      await unlink(stalePath)
      console.log(`removed stale generated icon ${stalePath}`)
    }
  }
}

for (const spec of specs) {
  const png = icons.get(spec.texture)
  if (png === undefined) throw new Error(`missing rendered icon: ${spec.texture}`)
  await writeOrCheck(resolve(outputRoot, `${spec.texture}.png`), png)
}

const currentXml = await readFile(texturesXmlPath, "utf8")
const expectedXml = updateTexturesXml(currentXml, textureBlock(specs), specs)
await writeOrCheck(texturesXmlPath, expectedXml)
await writeOrCheck(contactSheetPath, renderContactSheet(specs, icons))

if (checkOnly) {
  const sourceLicense = await readFile(sourceLicensePath)
  await writeOrCheck(licensePath, sourceLicense)
} else {
  await copyFile(sourceLicensePath, licensePath)
}

console.log(
  `${checkOnly ? "verified" : "generated"} ${specs.length} Inventory Explorer icons`
)
