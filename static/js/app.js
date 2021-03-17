
// Vue.component('status-bar', {
//   template: `
//   <div class="StatusBar">I am the status bar<span>
//   `
// })

const Main = {
  data() {
    return {
      status: {
        hostname: "unknown"
      },
      loading: true
    }
  },
  mounted() {
    axios.get('/api/status').then(response => {
      console.log("Got response", response)
      this.loading = false
      this.status = response.data
    })
  }
}

const app = Vue.createApp(Main)

app.component('status-bar', {
  props: ['status'],
  template: `<div class="StatusBar">Host: {{status.hostname}}</div>`
});

app.mount('#app')

// var app = new Vue({
//   el: "#app",
//   template: `
//     <status-bar>
//     Hello!
//   `
// })
