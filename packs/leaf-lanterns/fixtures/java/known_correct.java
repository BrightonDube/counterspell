// Test-only known-correct Java fixture for leaf-lanterns.
public class Solution {
    public static long count_leaves(long[] vals, long[] left, long[] right) {
        int n = vals.length;
        java.util.Set<Long> seen = new java.util.HashSet<>();
        java.util.ArrayDeque<Integer> stack = new java.util.ArrayDeque<>();
        if (n > 0) stack.push(0);
        long leaves = 0;
        while (!stack.isEmpty()) {
            int i = stack.pop();
            if (seen.contains((long) i)) continue;
            seen.add((long) i);
            long lo = i < left.length && left[i] >= 0 && left[i] < n ? left[i] : -1;
            long hi = i < right.length && right[i] >= 0 && right[i] < n ? right[i] : -1;
            if (lo == -1 && hi == -1) { leaves += 1; continue; }
            if (lo != -1) stack.push((int) lo);
            if (hi != -1) stack.push((int) hi);
        }
        return leaves;
    }
}
