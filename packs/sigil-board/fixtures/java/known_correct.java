// Test-only known-correct Java fixture for sigil-board.
public class Solution {
    static long f(int n, int r, java.util.List<Integer> cols) {
        if (r == n) {
            return 1;
        }
        long total = 0;
        for (int c = 0; c < n; c += 1) {
            if (cols.contains(c)) {
                continue;
            }
            boolean clash = false;
            for (int rr = 0; rr < cols.size(); rr += 1) {
                if (Math.abs(r - rr) == Math.abs(c - cols.get(rr))) {
                    clash = true;
                    break;
                }
            }
            if (!clash) {
                java.util.List<Integer> next = new java.util.ArrayList<>(cols);
                next.add(c);
                total += f(n, r + 1, next);
            }
        }
        return total;
    }
    public static long sigil_count(long n) {
        return f((int) n, 0, new java.util.ArrayList<>());
    }
}
