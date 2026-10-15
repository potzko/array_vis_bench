# `bubble_sort_lib`

Bubble sort family: bubble, shaker and recursive bubble sort with no axes, plus odd even bubble sort over a non trivial small sort.

Catalog fragment `bubble.spec`:

| component | type | label | provides | category |
|---|---|---|---|---|
| `bubble_sort` | `BubbleSort` | `bubble sort` | Sort | Sort |
| `shaker_sort` | `ShakerSort` | `shaker sort` | Sort | Sort |
| `bubble_sort_recursive` | `BubbleSortRecursive` | `bubble sort recursive` | Sort | Sort |
| `odd_even_bubble_sort` | `OddEvenBubbleSort<{small}>` | `odd-even bubble sort<small: {small}>` | Sort | Sort |

Query `bubble.avbs` builds: `BubbleSorts`, `ShakerSorts`, `BubbleSortsRecursive`, `OddEvenBubbleSorts`.
