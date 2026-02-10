/* debot gateway frontend logic (Alpine.js) */

document.addEventListener('alpine:init', () => {

  Alpine.data('gateway', () => ({
    tab: 'dashboard',
    // Status
    status: null,
    cronJobs: [],
    // Config sections
    config: null,
    // Toast
    toastMsg: '',
    toastTimer: null,
    // Auto-refresh
    refreshInterval: null,

    init() {
      this.fetchStatus();
      this.fetchConfig();
      this.startAutoRefresh();
    },

    destroy() {
      this.stopAutoRefresh();
    },

    startAutoRefresh() {
      this.refreshInterval = setInterval(() => {
        if (this.tab === 'dashboard') {
          this.fetchStatus();
        }
      }, 10000);
    },

    stopAutoRefresh() {
      if (this.refreshInterval) {
        clearInterval(this.refreshInterval);
      }
    },

    async fetchStatus() {
      try {
        const [statusRes, cronRes] = await Promise.all([
          fetch('/api/status'),
          fetch('/api/status/cron'),
        ]);
        this.status = await statusRes.json();
        const cronData = await cronRes.json();
        this.cronJobs = cronData.jobs || [];
      } catch (e) {
        console.error('Failed to fetch status', e);
      }
    },

    async fetchConfig() {
      try {
        const res = await fetch('/api/config');
        this.config = await res.json();
      } catch (e) {
        console.error('Failed to fetch config', e);
      }
    },

    async saveSection(section) {
      try {
        const res = await fetch(`/api/config/${section}`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(this.config[section]),
        });
        if (!res.ok) throw new Error(await res.text());
        const updated = await res.json();
        this.config[section] = updated;
        this.toast(`${section} saved`);
      } catch (e) {
        this.toast(`Error: ${e.message}`);
      }
    },

    toast(msg) {
      this.toastMsg = msg;
      if (this.toastTimer) clearTimeout(this.toastTimer);
      this.toastTimer = setTimeout(() => { this.toastMsg = ''; }, 3000);
    },

    /** Check if a value is a masked secret (contains ****) */
    isMasked(value) {
      return typeof value === 'string' && value.includes('****');
    },

    /** Get provider names as an array for x-for iteration */
    providerNames() {
      if (!this.config || !this.config.providers) return [];
      return Object.keys(this.config.providers);
    },

    formatUptime(s) {
      if (!s && s !== 0) return '-';
      const h = Math.floor(s / 3600);
      const m = Math.floor((s % 3600) / 60);
      const sec = Math.floor(s % 60);
      if (h > 0) return `${h}h ${m}m`;
      if (m > 0) return `${m}m ${sec}s`;
      return `${sec}s`;
    },

    formatTime(ms) {
      if (!ms) return '-';
      return new Date(ms).toLocaleString();
    },

    channelList() {
      if (!this.status || !this.status.channels) return [];
      return Object.entries(this.status.channels).map(([name, info]) => ({
        name,
        ...info,
      }));
    },
  }));
});
