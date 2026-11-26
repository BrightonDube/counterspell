// Test-only known-correct Java fixture for glow-sum.
public class Solution {
    public static long tree_sum(long[] vals, long[] left, long[] right) {
        int n = vals.length;
        java.util.Set<Integer> seen = new java.util.HashSet<>();
        java.util.ArrayDeque<Integer> stack = new java.util.ArrayDeque<>();
        if (n > 0) stack.push(0);
        long total = 0;
        while (!stack.isEmpty()) {
            int i = stack.pop();
            if (seen.contains(i)) continue;
            seen.add(i);
            total += vals[i];
            long lo = i < left.length && left[i] >= 0 && left[i] < n ? left[i] : -1;
            long hi = i < right.length && right[i] >= 0 && right[i] < n ? right[i] : -1;
            if (lo != -1) stack.push((int) lo);
            if (hi != -1) stack.push((int) hi);
        }
        return total;
    }
}
