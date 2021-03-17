
Vue.component('status-bar', {
  template: `
  <div class="StatusBar">I am the status bar<span>
  `
})

var app = new Vue({
  el: "#app",
  template: `
    <status-bar>
    Hello!
  `
})
