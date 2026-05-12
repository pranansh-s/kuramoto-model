import csv
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

K_vals = []
r_vals = []

with open('data.csv', 'r') as f:
    reader = csv.DictReader(f)
    for row in reader:
        K_vals.append(float(row['K']))
        r_vals.append(float(row['r']))

fig, ax = plt.subplots(figsize=(10, 6))
ax.plot(K_vals, r_vals, 'o-', color='#4A90D9', markersize=4, linewidth=1.5, label='Simulation (N=200)')
ax.set_xlabel('Coupling Strength K', fontsize=14)
ax.set_ylabel('Order Parameter r', fontsize=14)
ax.set_title('Kuramoto Model: Synchronisation Order vs Coupling Strength', fontsize=16)
ax.set_ylim(-0.02, 1.05)
ax.set_xlim(-0.1, max(K_vals) + 0.1)
ax.axhline(y=0, color='gray', linestyle='--', alpha=0.3)
ax.legend(fontsize=12)
ax.grid(True, alpha=0.3)
fig.tight_layout()
fig.savefig('kuramoto_r_vs_K.png', dpi=150)
print("Plot saved to kuramoto_r_vs_K.png")
