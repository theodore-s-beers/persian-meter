import statistics
from collections import Counter
from pathlib import Path


def main() -> None:
    base_dir = Path(".")
    ghazal_dirs = ["hafiz-1", "hafiz-2"]

    line_counts: list[tuple[str, int]] = []

    for dir_name in ghazal_dirs:
        dir_path = base_dir / dir_name
        if not dir_path.exists():
            raise ValueError(f"Directory {dir_path} does not exist")

        txt_files = sorted(
            dir_path.glob("*.txt"),
            key=lambda x: int(x.stem) if x.stem.isdigit() else float("inf"),
        )

        for txt_file in txt_files:
            count = count_nonempty_lines(txt_file)
            line_counts.append((txt_file.name, count))
            print(f"{txt_file.name}: {count} lines")

    if line_counts:
        print("\n" + "=" * 50)
        print("STATISTICS")
        print("=" * 50)

        counts_only = [count for _, count in line_counts]

        print(f"Total ghazals: {len(line_counts)}")
        print(f"Mean lines per ghazal: {statistics.mean(counts_only):.2f}")
        print(f"Median lines per ghazal: {statistics.median(counts_only):.2f}")
        print(f"Min lines: {min(counts_only)}")
        print(f"Max lines: {max(counts_only)}")
        print(f"Standard deviation: {statistics.stdev(counts_only):.2f}")

        print("\nDistribution:")

        distribution = Counter(counts_only)
        for num_lines in sorted(distribution.keys()):
            print(f"  {num_lines} lines: {distribution[num_lines]} ghazals")


def count_nonempty_lines(filepath: Path) -> int:
    with open(filepath, "r", encoding="utf-8") as f:
        total = sum(1 for line in f if line.strip())
        if total % 2 != 0:
            raise ValueError(
                f"File {filepath} has an odd number of hemistichs: {total}"
            )
        if total > 28:
            print(f"File {filepath} has a large number of hemistichs: {total}")
        return total // 2


if __name__ == "__main__":
    main()
