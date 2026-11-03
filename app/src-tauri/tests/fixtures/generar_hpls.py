# Genera ejemplo.hpls: una lista de Holyrics de prueba con letras inventadas, con la misma estructura que un
# .hpls real (registros [tipo][largo][objeto Java serializado] con com.limagiran.holyrics.model.Music).
# Uso: python3 generar_hpls.py   (escribe ejemplo.hpls junto a este archivo)
import os
import struct

MUSIC_FIELDS = [
    ("Z", "archived"), ("J", "id"), ("J", "modifiedTime"), ("J", "set"), ("J", "version"),
    ("L", "artist"), ("L", "artistHTML"), ("L", "artistToSearch"), ("L", "author"), ("L", "formatting"),
    ("L", "language"), ("L", "lyrics"), ("L", "lyricsHTML"), ("L", "lyricsToSearch"), ("L", "midi"),
    ("L", "note"), ("L", "noteToSearch"), ("L", "numIni"), ("L", "order"), ("L", "src"),
    ("[", "strParams"), ("L", "title"), ("L", "titleHTML"), ("L", "titleToSearch"), ("[", "translations"),
]
TYPES = {
    "numIni": "Ljava/lang/Integer;", "order": "Ljava/util/List;",
    "strParams": "[Ljava/lang/String;", "translations": "[[Ljava/lang/String;",
}


def mutf8(text):
    out = bytearray()
    for unit in text.encode("utf-16-be").hex(" ", 2).split():
        u = int(unit, 16)
        if 0 < u < 0x80:
            out.append(u)
        elif u < 0x800:
            out += bytes([0xC0 | u >> 6, 0x80 | u & 0x3F])
        else:
            out += bytes([0xE0 | u >> 12, 0x80 | (u >> 6) & 0x3F, 0x80 | u & 0x3F])
    return bytes(out)


class Writer:
    def __init__(self):
        self.out = bytearray(b"\xac\xed\x00\x05")
        self.handles = {}
        self.next = 0x7E0000

    def handle(self, key):
        self.handles[key] = self.next
        self.next += 1

    def string(self, text):
        if ("s", text) in self.handles:
            self.out += b"\x71" + struct.pack(">i", self.handles[("s", text)])
            return
        data = mutf8(text)
        self.out += b"\x74" + struct.pack(">H", len(data)) + data
        self.handle(("s", text))

    def classdesc(self, name, suid, flags, fields, parent=None):
        if ("c", name) in self.handles:
            self.out += b"\x71" + struct.pack(">i", self.handles[("c", name)])
            return
        data = mutf8(name)
        self.out += b"\x72" + struct.pack(">H", len(data)) + data + bytes.fromhex(suid)
        self.handle(("c", name))
        self.out += bytes([flags]) + struct.pack(">H", len(fields))
        for kind, field, type_name in fields:
            f = mutf8(field)
            self.out += kind.encode() + struct.pack(">H", len(f)) + f
            if kind in "L[":
                self.string(type_name)
        self.out += b"\x78"
        if parent:
            parent()
        else:
            self.out += b"\x70"

    def integer(self, value):
        self.out += b"\x73"
        self.classdesc("java.lang.Integer", "12e2a0a4f7818738", 0x02, [("I", "value", None)],
                       lambda: self.classdesc("java.lang.Number", "86ac951d0b94e08b", 0x02, []))
        self.handle(("o", id(value)))
        self.out += struct.pack(">i", value)

    def array_list(self, items):
        self.out += b"\x73"
        self.classdesc("java.util.ArrayList", "7881d21d99c7619d", 0x03, [("I", "size", None)])
        self.handle(("o", "lista"))
        self.out += struct.pack(">i", len(items))
        self.out += b"\x77\x04" + struct.pack(">i", len(items))
        for item in items:
            self.string(item)
        self.out += b"\x78"

    def string_array(self, items):
        self.out += b"\x75"
        self.classdesc("[Ljava.lang.String;", "adb257e7e91d7b47", 0x02, [])
        self.handle(("o", "arreglo"))
        self.out += struct.pack(">i", len(items))
        for item in items:
            if item is None:
                self.out += b"\x70"
            else:
                self.string(item)


def music(title, artist, author, lyrics, order):
    w = Writer()
    w.out += b"\x73"
    fields = [(k, f, TYPES.get(f, "Ljava/lang/String;")) for k, f in MUSIC_FIELDS]
    w.classdesc("com.limagiran.holyrics.model.Music", "00000000000030cb", 0x02, fields)
    w.handle(("o", "music"))
    values = {"artist": artist, "author": author, "lyrics": lyrics, "title": title, "src": "new", "note": ""}
    for kind, field in MUSIC_FIELDS:
        if kind == "Z":
            w.out += b"\x00"
        elif kind == "J":
            w.out += struct.pack(">q", 1696727162763 if field == "set" else 2)
        elif field == "numIni":
            w.integer(0)
        elif field == "order":
            w.array_list(order)
        elif field == "strParams":
            w.string_array([None, None, ""])
        elif field == "translations":
            w.out += b"\x70"
        elif field in values:
            w.string(values[field])
        elif field == "titleToSearch":
            w.string(title.lower())
        else:
            w.out += b"\x70"
    return bytes(w.out)


def record(kind, payload):
    return struct.pack(">ii", kind, len(payload)) + payload


songs = [
    music("Canción de prueba", "Coro de ejemplo", "",
          "Primera estrofa\ncon dos líneas\n\n\nCoro: ¡aleluya! ñandú  \n \nFinal\n", []),
    music("  Segunda  ", "Banda Inventada", "Autora Inventada", "Uno\n\nDos", ["1", "2"]),
    music("Himno sin párrafos", "", "", "Una sola estrofa\nde un himno\nsin separar", []),
]
tail = Writer()
tail.string("")
data = b"".join(record(0, s) for s in songs) + record(3, bytes(tail.out))
path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "ejemplo.hpls")
with open(path, "wb") as f:
    f.write(data)
print(path, len(data), "bytes")
