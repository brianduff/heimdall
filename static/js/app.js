const Main = {
  data() {
    return {
      status: {
        hostname: "unknown",
        is_configured: false
      },
      loading: true
    }
  },
  mounted() {
    axios.get('/api/status').then(response => {
      this.loading = false
      this.status = response.data
    })
  }
}

const app = Vue.createApp(Main)

app.component('loading-bar', {
  template: `<div>Loading...</div>`
})

app.component('status-bar', {
  props: ['status'],
  template: `<div class="StatusBar">Computer: {{status.hostname}}</div>`
})

app.component('first-time-setup', {
  template: `
    <h1>Welcome to Heimdall!</h1>
    <p>You've never used Heimdall before on this computer. Let's get it set up!</p>
  `
})

app.mount('#app')
