// Test-only known-correct Java fixture for frame-rite.
public class Solution {
    public static long frame_rite(long[] pages, long f) {
        java.util.ArrayList<Long> frame = new java.util.ArrayList<>();
        long faults = 0;
        int n = pages.length;
        for (int i = 0; i < n; i++) {
            long p = pages[i];
            if (frame.contains(p)) continue;
            faults += 1;
            if (frame.size() < f) { frame.add(p); continue; }
            int worst = 0, worstNext = -1;
            for (int k = 0; k < frame.size(); k++) {
                int nxt = n + 1;
                for (int j = i + 1; j < n; j++) if (pages[j] == frame.get(k)) { nxt = j; break; }
                if (nxt > worstNext) { worst = k; worstNext = nxt; }
            }
            frame.set(worst, p);
        }
        return faults;
    }
}
