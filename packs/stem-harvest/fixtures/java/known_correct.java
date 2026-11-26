// Test-only known-correct Java fixture for stem-harvest.
public class Solution {
    public static long stem_depth_sum(String words) {
        java.util.Set<String> ws = new java.util.HashSet<>();
        for (String w : words.split(",", -1)) {
            if (!w.isEmpty()) {
                ws.add(w);
            }
        }
        long ans = 0;
        for (String w : ws) {
            int best = 0;
            for (String o : ws) {
                if (!o.equals(w) && w.startsWith(o) && o.length() > best) {
                    best = o.length();
                }
            }
            ans += best;
        }
        return ans;
    }
}
