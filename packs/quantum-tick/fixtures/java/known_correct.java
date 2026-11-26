// Test-only known-correct Java fixture for quantum-tick.
public class Solution {
    public static long quantum_tick(long[][] jobs, long q) {
        Integer[] order = new Integer[jobs.length];
        for (int i = 0; i < jobs.length; i++) order[i] = i;
        java.util.Arrays.sort(order, (x, y) -> jobs[x][0] != jobs[y][0] ? Long.compare(jobs[x][0], jobs[y][0]) : x - y);
        long[] rem = new long[jobs.length];
        for (int i = 0; i < jobs.length; i++) rem[i] = jobs[i][1];
        java.util.Deque<Integer> ready = new java.util.ArrayDeque<>();
        int i = 0, n = order.length, done = 0;
        long t = 0, total = 0;
        while (done < n) {
            while (i < n && jobs[order[i]][0] <= t) { ready.addLast(order[i]); i += 1; }
            if (ready.isEmpty()) { t = jobs[order[i]][0]; continue; }
            int j = ready.pollFirst();
            long run = Math.min(q, rem[j]);
            t += run;
            rem[j] -= run;
            while (i < n && jobs[order[i]][0] <= t) { ready.addLast(order[i]); i += 1; }
            if (rem[j] > 0) ready.addLast(j);
            else { total += t; done += 1; }
        }
        return total;
    }
}
