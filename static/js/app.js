const bus = mitt()

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
  methods: {
    selectUser: user => bus.emit('setup-choose-user', user)
  },
  template: `<button @click="selectUser(user)" class="UserTile"><img src="img/user.svg" width="30"><span>{{ user.realname }}</span></button>`
})

app.component('setup-choose-user', {
  template: `
    <div>
      <p>You've never used Heimdall before on this computer. Let's get it set up!</p>
      <p>Please select a user to install Heimdall for:</p>
      <ul class="UserSelector">
        <div v-for="user in users"><user-tile :user="user" /></div>
      </ul>
    </div>
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

app.component('setup-configure-passwords', {
  props: ['user'],
  methods: {
    passwordChanged() {
      var passwordsMatch = false
      const validation = document.getElementById("validation")
      if (this.lockdownPassword !== this.lockdownPasswordRepeat) {
        validation.style.visibility = "visible"
        validation.innerText = "Lockdown passwords do not match"
      } else {
        validation.style.visibility = "hidden"
        passwordsMatch = true
      }

      this.setNextButtonEnabled(passwordsMatch && this.lockdownPassword.trim() && this.lockdownPasswordRepeat.trim() && this.currentPassword.trim())

    },
    setNextButtonEnabled(enabled) {
      document.getElementById("nextButton").disabled = !enabled
    }
  },
  data() {
    return {
      currentPassword: "",
      lockdownPassword: "",
      lockdownPasswordRepeat: "",
    }
  },
  template: `
    <div>
      <p>Heimdall will prevent {{ user.realname }} from logging into their computer by changing their password whenever their computer is locked down.</p>
      <p>To do this, Heimdall needs to know the current password of {{ user.realname }} and a lockdown password you'd like to use for {{ user.realname }}.
      You should keep the lockdown password secret from {{ user.realname }}.</p>
      <div class="PasswordForm">
        <span>Current Password:</span><input id="currentPassword" type="password" v-model="currentPassword" @keyup="passwordChanged" />
        <span>New lockdown password:</span><input id="lockdownPassword" type="password" v-model="lockdownPassword" @keyup="passwordChanged" />
        <span>Repeat new lockdown password:</span><input id="lockdownPasswordRepeat" type="password" v-model="lockdownPasswordRepeat" @keyup="passwordChanged" />
        <span/><span class="Validation" id="validation" style="visibility:hidden">Form has no errors</span>
        <span /><div><button id="nextButton">Next</button></div>
      </div>
      <p>For security, these passwords will be stored in the Mac OS Keychain.</p>
    </div>
  `,
  mounted() {
    document.getElementById("currentPassword").focus()
    this.setNextButtonEnabled(false)
  }
})

app.component('first-time-setup', {
  template: `
    <h1>Welcome to Heimdall!</h1>
    <setup-choose-user v-if="stage === 'GET_USER_NAME'"></setup-choose-user>
    <setup-configure-passwords v-if="stage === 'PASSWORD_CONFIG'" :user="this.user" />
  `,
  data() {
    return {
      stage: "GET_USER_NAME",
      user: null,
    }
  },
  mounted() {
    bus.on("setup-choose-user", user => {
      this.user = user
      this.stage = "PASSWORD_CONFIG"
    })
  }
})

app.mount('#app')
