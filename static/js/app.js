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
      <ul class="UserSelector Form">
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
    },
    submitPasswords() {
      bus.emit('setup-passwords-entered', { currentPassword: this.currentPassword, lockdownPassword: this.lockdownPassword })
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
      <div class="PasswordForm Form">
        <span>Current Password:</span><input id="currentPassword" type="password" v-model="currentPassword" @keyup="passwordChanged" />
        <span>New lockdown password:</span><input id="lockdownPassword" type="password" v-model="lockdownPassword" @keyup="passwordChanged" />
        <span>Repeat new lockdown password:</span><input id="lockdownPasswordRepeat" type="password" v-model="lockdownPasswordRepeat" @keyup="passwordChanged" />
        <span/><span class="Validation" id="validation" style="visibility:hidden">Form has no errors</span>
        <span /><div><button id="nextButton" @click="submitPasswords">Next</button></div>
      </div>
      <p>For security, these passwords will be stored in the Mac OS Keychain.</p>
    </div>
  `,
  mounted() {
    document.getElementById("currentPassword").focus()
    this.setNextButtonEnabled(false)
  }
})

app.component('setup-configure-schedule', {
  props: ["user"],
  template: `
    <div>
      <p>Heimdall can lock down {{ user.realname }}'s access on a schedule. By default, the computer will be locked down except for unlocked periods that you specify here.</p>
      <p>Add as many unlock periods as you like, and click Done when you're finished. You can also edit these later.</p>
      <div class="Form">
        <div v-for="(spec, index) in this.specs">
          <schedule-period :spec="spec" :index="index" :itemcount="this.specs.length" />
        </div>
        <button @click="submitSchedule">Next</button>
      </div>
    </div>
  `,
  data() {
    return {
      specs: [
        {day: 'Sunday', startTime: '540', endTime: '570', duration: 30, note: ""}
      ]
    }
  },
  mounted() {
    bus.on("add-schedule-item", () => {
      this.specs.push({day: 'Sunday', startTime: '540', endTime: '570', duration: 30, note: ""})
    })
    bus.on("remove-schedule-item", ({ index }) => {
      this.specs.splice(index, 1)
    })
  },
  methods: {
    submitSchedule() {
      bus.emit('schedule-complete', this.specs)
    }
  }
})

app.component('schedule-period', {
  props: ["spec", "index", "itemcount"],
  template: `
    <div class="SchedulePeriod">
      <select id="day" name="day" v-model="spec.day">
        <option value="Sunday">Sunday</option>
        <option value="Monday">Monday</option>
        <option value="Tuesday">Tuesday</option>
        <option value="Wednesday">Wednesday</option>
        <option value="Thursday">Thursday</option>
        <option value="Friday">Friday</option>
        <option value="Saturday">Saturday</option>
      </select>
      <select v-bind:id="'start' + index" name="start" v-model="spec.startTime" @change="changeStartTime">
      </select>
      <span>&mdash;</span>
      <select v-bind:id="'end' + index" name="end" v-model="spec.endTime" @change="changeEndTime">
      </select>
      <input placeholder="Note for this period" v-model="spec.note" />
      <button @click="removeScheduleItem(index)" v-bind:class="{ 'Hidden': itemcount == 1 }" class="IconButton"><i class="fas fa-minus-square"></i></button>
      <button @click="addScheduleItem" v-bind:class="{ 'Hidden': index !== itemcount -1 }" class="IconButton"><i class="fas fa-plus-square"></i></button>
    </div>
  `,
  methods: {
    addScheduleItem() {
      bus.emit("add-schedule-item")
    },
    removeScheduleItem(index) {
      bus.emit("remove-schedule-item", { index })
    },
    changeStartTime() {
      // Regenerate the end times
      this.generateSelectTimes("end", this.spec.startTime)
      // Update the selected end time to maintain the duration
      const newEndTime = parseInt(this.spec.startTime, 10) + this.spec.duration
      this.spec.endTime = newEndTime.toString()
    },
    changeEndTime() {
      this.duration = parseInt(this.spec.endTime) - parseInt(this.spec.startTime)
    },
    generateSelectTimes(id, startTime) {
      const hasStartTime = startTime
      startTime = startTime ? parseInt(startTime, 10) : 0;

      const s = document.getElementById(id + this.index)

      // Clear all existing things
      for (var i = s.options.length - 1; i >= 0; i--) {
        s.remove(i)
      }

      for (var time = startTime + 15; time < 1440; time += 15) {
        let ampm = "am"
        let hour = Math.floor(time / 60)
        if (hour == 0) {
          hour = 12
        } else if (hour >= 12) {
          ampm = "pm"
          if (hour >= 13) {
            hour -= 12
          }
        }
        let minute = time % 60
        minute = (minute < 10) ? ("0" + minute) : minute

        const durationHours = Math.floor((time - startTime) / 60)
        const durationMinutes = (time - startTime) % 60

        var durationString = ""
        if (hasStartTime) {
          if (durationHours == 0) {
            durationString = ` (${durationMinutes} mins)`
          } else if (durationMinutes == 0) {
            if (durationHours == 1) {
              durationString = " (1 hr)"
            } else {
              durationString = ` (${durationHours} hrs)`
            }
          } else {
            let durationFraction = ""
            switch (durationMinutes) {
              case 15:
                durationFraction = "25"
                break;
              case 30:
                durationFraction = "5"
                break;
              case 45:
                durationFraction = "75"
                break;
            }
            durationString = ` (${durationHours}.${durationFraction} hrs)`
          }
        }

        const option = document.createElement("option")
        option.value = `${time}`
        option.text = `${hour}:${minute}${ampm}${durationString}`
        s.appendChild(option)
      }
    }
  },
  mounted() {
    this.generateSelectTimes("start", null)
    document.getElementById("start" + this.index).value = this.spec.startTime
    this.generateSelectTimes("end", this.spec.startTime)
    document.getElementById("end" + this.index).value = this.spec.endTime
  }
})

app.component('first-time-setup', {
  template: `
    <h1>Welcome to Heimdall!</h1>
    <setup-choose-user v-if="stage === 'GET_USER_NAME'" />
    <setup-configure-passwords v-if="stage === 'PASSWORD_CONFIG'" :user="this.user" />
    <setup-configure-schedule v-if="stage === 'SCHEDULE_CONFIG'" :user="this.user" />
    <div v-if="stage === 'READY_TO_ENABLE'">
      <div>
        Heimdall is ready to be configured for {{ this.user.realname }}. Click the button below to switch it on.
      </div>
      <button id="configureSubmit" @click="configureSubmit">Enable Hiemdall for {{ this.user.realname }}</button>
    </div>
  `,
  data() {
    return {
      stage: "GET_USER_NAME",
      user: null,
      passwords: null,
      schedule: null,
    }
  },
  mounted() {
    bus.on("setup-choose-user", user => {
      this.user = user
      this.stage = "PASSWORD_CONFIG"
    })
    bus.on("setup-passwords-entered", passwords => {
      this.passwords = passwords
      this.stage = "SCHEDULE_CONFIG"
    })
    bus.on("schedule-complete", specs => {
      this.schedule = specs
      this.stage = "READY_TO_ENABLE"
    })
  },
  methods: {
    configureSubmit() {
      document.getElementById("configureSubmit").disabled = true

      let request = {
        username: this.user.username,
        normal_password: this.passwords.currentPassword,
        lockdown_password: this.passwords.lockdownPassword,
        schedule: {
          open_periods: []
        }
      }

      let toHour = time => Math.floor(parseInt(time, 10) / 60)
      let toMinute = time => parseInt(time, 10) % 60

      this.schedule.forEach(item => {
        let weekday = [ "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday" ].indexOf(item.day)
        let period = {
          start: {
            weekday,
            hour: toHour(item.startTime),
            minute: toMinute(item.startTime)
          },
          end: {
            weekday,
            hour: toHour(item.endTime),
            minute: toMinute(item.endTime)
          },
          note: item.note
        }
        request.schedule.open_periods.push(period)
      })

      axios.post('/api/userconfig/', request).then((resp) => {
        console.log("Ok")
      })
    }
  }
})

app.mount('#app')
