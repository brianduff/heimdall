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

app.component('user-tile', {
  props: ['user'],
  template: `<div><img v-bind:src="'data:' + user.picture_mimetype + ';base64, ' + user.picture_base64" /><span>{{ user.realname }}</span></div>`
})

app.component('first-time-setup', {
  template: `
    <h1>Welcome to Heimdall!</h1>
    <p>You've never used Heimdall before on this computer. Let's get it set up!</p>
    <p>Please select a user to install Heimdall for:</p>
    <ul>
      <li v-for="user in users"><user-tile :user="user" /></li>
    </ul>
  `,
  data() {
    return {
      users: []
    }
  },
  mounted() {
    axios.get("/api/users").then(response => {
      this.users = response.data
    })
  }
})

app.mount('#app')
