// Test-only known-correct Java fixture for bst-ward.
public class Solution {
    public static long bst_check(long[] vals, long[] left, long[] right) {
        int n = vals.length;
        if (n == 0) return 1;
        java.util.Set<String> seen = new java.util.HashSet<>();
        java.util.ArrayDeque<long[]> work = new java.util.ArrayDeque<>();
        work.push(new long[]{0, Long.MIN_VALUE / 2, Long.MAX_VALUE / 2});
        while (!work.isEmpty()) {
            long[] cur = work.pop();
            int i = (int) cur[0];
            if (!(cur[1] < vals[i] && vals[i] < cur[2])) return 0;
            String key = i + "," + cur[1] + "," + cur[2];
            if (seen.contains(key)) continue;
            seen.add(key);
            long l = i < left.length && left[i] >= 0 && left[i] < n ? left[i] : -1;
            long r = i < right.length && right[i] >= 0 && right[i] < n ? right[i] : -1;
            if (l != -1) work.push(new long[]{l, cur[1], vals[i]});
            if (r != -1) work.push(new long[]{r, vals[i], cur[2]});
        }
        return 1;
    }
}
