// Test-only known-correct Java fixture for merge-glow.
public class Solution {
    public static long[] merge_glow(long[] am, long[] an, long[] bm, long[] bn) {
        java.util.List<Long> out = new java.util.ArrayList<>();
        long i = an.length > 0 ? 0 : -1, j = bn.length > 0 ? 0 : -1;
        while (i != -1 && j != -1) {
            if (am[(int) i] <= bm[(int) j]) { out.add(am[(int) i]); i = an[(int) i]; }
            else { out.add(bm[(int) j]); j = bn[(int) j]; }
        }
        for (; i != -1; i = an[(int) i]) { out.add(am[(int) i]); }
        for (; j != -1; j = bn[(int) j]) { out.add(bm[(int) j]); }
        return out.stream().mapToLong(Long::longValue).toArray();
    }
}
