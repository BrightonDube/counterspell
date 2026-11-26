// Test-only known-correct Java fixture for burst-rite.
public class Solution {
    public static long burst_rite(long[][] jobs) {
        Integer[] order = new Integer[jobs.length];
        for (int i = 0; i < jobs.length; i++) order[i] = i;
        java.util.Arrays.sort(order, (x, y) -> jobs[x][0] != jobs[y][0] ? Long.compare(jobs[x][0], jobs[y][0]) : x - y);
        long clock = 0, total = 0;
        for (int i : order) {
            long a = jobs[i][0], b = jobs[i][1];
            long start = Math.max(clock, a);
            total += start - a;
            clock = start + b;
        }
        return total;
    }
}
