// Test-only known-correct Java fixture for page-turn.
public class Solution {
    public static long page_turn(long[] pages, long f) {
        java.util.LinkedList<Long> folio = new java.util.LinkedList<>();
        long faults = 0;
        for (long p : pages) {
            if (!folio.contains(p)) {
                faults += 1;
                if (folio.size() == f) folio.pollFirst();
                folio.addLast(p);
            }
        }
        return faults;
    }
}
