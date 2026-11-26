// Test-only known-correct Java fixture for kindred-path.
public class Solution {
    public static long ember_path(long[] vals, long[] left, long[] right, long target) {
        int n = vals.length;
        if (n == 0) return 0;
        java.util.Set<String> seen = new java.util.HashSet<>();
        java.util.ArrayDeque<long[]> work = new java.util.ArrayDeque<>();
        work.push(new long[]{0, vals[0]});
        while (!work.isEmpty()) {
            long[] cur = work.pop();
            int i = (int) cur[0];
            long total = cur[1];
            String key = i + "," + total;
            if (seen.contains(key)) continue;
            seen.add(key);
            long lo = i < left.length && left[i] >= 0 && left[i] < n ? left[i] : -1;
            long hi = i < right.length && right[i] >= 0 && right[i] < n ? right[i] : -1;
            if (lo == -1 && hi == -1) {
                if (total == target) return 1;
                continue;
            }
            if (lo != -1) work.push(new long[]{lo, total + vals[(int) lo]});
            if (hi != -1) work.push(new long[]{hi, total + vals[(int) hi]});
        }
        return 0;
    }
}
