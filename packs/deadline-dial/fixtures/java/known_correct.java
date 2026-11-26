// Test-only known-correct Java fixture for deadline-dial.
public class Solution {
    public static long dial_count(long[] slots) {
        int n = slots.length / 2;
        long[][] jobs = new long[n][2];
        for (int i = 0; i < n; i += 1) {
            jobs[i][0] = slots[2 * i];
            jobs[i][1] = slots[2 * i + 1];
        }
        java.util.Arrays.sort(jobs, (a, b) -> Long.compare(a[0], b[0]));
        java.util.PriorityQueue<Long> held =
            new java.util.PriorityQueue<>(java.util.Collections.reverseOrder());
        long time = 0;
        for (long[] job : jobs) {
            time += job[1];
            held.add(job[1]);
            if (time > job[0]) {
                time -= held.poll();
            }
        }
        return held.size();
    }
}
