// Auto-generated sort registrations
// Add this to your main.rs or lib.rs

use crate::traits::{init_sort_registry, register_sort};

pub fn register_all_sorts() {
    init_sort_registry();

    register_sort("shell sort 2.25 shrink factor", "O(N^2)", false, "shell_shell_sorts");
    register_sort("shell shell sort optimised", "O(N^2)", false, "shell_shell_sorts");
    register_sort("shell shell sort log parity", "O(N^2)", false, "shell_shell_sorts");
    register_sort("shell shell sort fibonacci", "O(N^2)", false, "shell_shell_sorts");
    register_sort("shell shell sort 3 parity", "O(N^2)", false, "shell_shell_sorts");
    register_sort("shell shell sort root parity", "O(N^2)", false, "shell_shell_sorts");
    register_sort("shell shell sort classic", "O(N^2)", false, "shell_shell_sorts");
    register_sort("shell sort classic", "O(N^2)", false, "classic_shell_sorts");
    register_sort("shell sort sedgewick branching", "O(N^(4/3))", false, "classic_shell_sorts");
    register_sort("shell sort hibbard", "O(N^(3/2))", false, "classic_shell_sorts");
    register_sort("shell sort classic ordered insertion", "O(N^2)", false, "classic_shell_sorts");
    register_sort("shell sort optimized 256 elements", "O(N^1.5)", false, "classic_shell_sorts");
    register_sort("shell sort classic dissonance", "O(N^2)", false, "classic_shell_sorts");
    register_sort("shell sort sedgewick", "O(N^(4/3))", false, "classic_shell_sorts");
    register_sort("shell sort knuth", "O(N^(3/2))", false, "classic_shell_sorts");
    register_sort("shell sort sedgewick ordered insertion", "O(N^(4/3))", false, "classic_shell_sorts");
    register_sort("comb sort classic", "O(N^2)", false, "comb_sorts");
    register_sort("comb sort random gaps", "O(N^2)", false, "comb_sorts");
    register_sort("quick sort left right pivot optimised", "O(N Log(N))", false, "quick_sorts");
    register_sort("quick sort left right pointers static pivot", "O(N Log(N))", false, "quick_sorts");
    register_sort("quick sort left left pointers", "O(N Log(N))", false, "quick_sorts");
    register_sort("median pivot quick sort", "O(N Log(N))", false, "quick_sorts");
    register_sort("quick sort left left pointers optimised", "O(N Log(N))", false, "quick_sorts");
    register_sort("iterative quick sort", "O(N Log(N))", false, "quick_sorts");
    register_sort("quick sort left right pointers moving pivot", "O(N Log(N))", false, "quick_sorts");
    register_sort("stooge circle sort", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("shaker circle sort bottom up b", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("shaker circle sort bottom up", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("stooge circle sort reversed", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("circle sort recursive increasing size", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("circle sort bottom up", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("circle sort bottom up optimised", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("circle sort bottom up increasing size", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("circle sort recursive optimised", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("shaker circle sort", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("circle sort recursive", "O(N^log^2(N))", false, "circle_sorts");
    register_sort("bubble sort", "O(N^2)", true, "bubble_sorts");
    register_sort("shaker sort", "O(N^2)", true, "bubble_sorts");
    register_sort("bubble sort recursive", "O(N^2)", true, "bubble_sorts");
    register_sort("odd-even bubble sort", "O(N^2)", true, "bubble_sorts");
    register_sort("insertion sort", "O(N^2)", true, "insertion_sorts");
    register_sort("cycle sort", "O(N^2)", false, "cycle_sorts");
    register_sort("slow sort", "O(N^3)", false, "fun_sorts");
    register_sort("quick surrender optimised", "O(N^2)", false, "fun_sorts");
    register_sort("bad heap sort", "O(N^2)", false, "fun_sorts");
    register_sort("stooge sort", "O(N^(logn))", false, "fun_sorts");
    register_sort("shell sort hibbard jumps", "O(N^2.5)", false, "fun_sorts");
    register_sort("bad heap sort alt", "O(N^2)", false, "fun_sorts");
    register_sort("cyclent sort", "O(N^3?)", false, "fun_sorts");
    register_sort("quick surrender", "O(N^2)", false, "fun_sorts");
    register_sort("cyclent sort stack optimized", "O(N^3?)", false, "fun_sorts");
    register_sort("cyclent sort stack", "O(N^3?)", false, "fun_sorts");
    register_sort("merge sort outside lists", "O(N Log(N))", true, "classic_merge_sorts");
    register_sort("merge sort", "O(N Log(N))", true, "classic_merge_sorts");
    register_sort("merge sort bottom up", "O(N Log(N))", true, "classic_merge_sorts");
    register_sort("merge sort bottom up optimized", "O(N Log(N))", true, "classic_merge_sorts");
    register_sort("merge sort optimized", "O(N Log(N))", true, "classic_merge_sorts");
    register_sort("rotate merge sort", "O(N Log(N)^2)", true, "rotate_merge_sorts");
    register_sort("rotate merge sort bottom up optimized", "O(N Log(N)^2)", true, "rotate_merge_sorts");
    register_sort("rotate merge sort bottom up", "O(N Log(N)^2)", true, "rotate_merge_sorts");
    register_sort("rotate merge sort optimized", "O(N Log(N)^2)", true, "rotate_merge_sorts");
    register_sort("heap sort base 3", "O(N*log(N))", false, "classic_heap_sorts");
    register_sort("heap sort classic", "O(N*log(N))", false, "classic_heap_sorts");
    register_sort("heap sort base 256", "O(N*log(N))", false, "classic_heap_sorts");
    register_sort("heap sort base 16", "O(N*log(N))", false, "classic_heap_sorts");
    register_sort("heap quick sort", "O(N*log(N))", false, "quick_heap_sorts");
    register_sort("heap quick sort optimized", "O(N*log(N))", false, "quick_heap_sorts");
    register_sort("heap quick sort optimized tmp", "O(N*log(N))", false, "quick_heap_sorts");
    register_sort("heap quick sort", "O(N*log(N))", false, "heap_sort");
    register_sort("weak heap sort", "O(N*log(N))", false, "heap_sort");
}
