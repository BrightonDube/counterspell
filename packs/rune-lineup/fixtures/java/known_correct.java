// Test-only known-correct Java fixture for rune-lineup.
public class Solution {
    static long go(Long prev, java.util.List<Long> left) {
        if (left.isEmpty()) {
            return 1;
        }
        long total = 0;
        for (int i = 0; i < left.size(); i += 1) {
            if (prev == null || !left.get(i).equals(prev)) {
                java.util.List<Long> rest = new java.util.ArrayList<>(left);
                long m = rest.remove(i);
                total += go(m, rest);
            }
        }
        return total;
    }
    public static long lineups(long[] marks) {
        java.util.List<Long> left = new java.util.ArrayList<>();
        for (long m : marks) {
            left.add(m);
        }
        return go(null, left);
    }
}
