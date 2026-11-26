// Test-only known-correct Java fixture for lru-rite.
public class Solution {
    public static long lru_rite(long[] pages, long f) {
        java.util.LinkedList<Long> folio = new java.util.LinkedList<>();
        long faults = 0;
        for (long p : pages) {
            if (folio.remove(p)) {
                folio.addLast(p);
            } else {
                faults += 1;
                if (folio.size() == f) folio.pollFirst();
                folio.addLast(p);
            }
        }
        return faults;
    }
}
