# Print one line per emitted catalog entry, `"<display name>"<TAB><concrete
# Rust type>`, sorted, from the generated `catalog_entries.rs`. `verify.sh`
# diffs this against `catalog.types`, so a solver or emit change that swaps
# a variant for another of the same name, which the golden cannot see, fails
# the gate.
import re
import sys
from pathlib import Path

ENTRY = re.compile(r"pub type Ty = (?P<ty>.*?);\s*const NAME: &str = (?P<name>\".*?\");", re.DOTALL)


def fingerprint(path: Path) -> list[str]:
    return sorted(f"{m['name']}\t{' '.join(m['ty'].split())}" for m in ENTRY.finditer(path.read_text()))


if __name__ == "__main__":
    print("\n".join(fingerprint(Path(sys.argv[1]))))
