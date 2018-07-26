import Home from './routes/Home';
import NewPost from './routes/NewPost';
import Inbox from './routes/Inbox';
import Chronometer from './routes/Chronometer';

import { request } from './util';

const MSG_POLL_INTERVAL = 10 * 1000;

const routes = [
  { name: 'Home', path: '/', component: Home },
  { name: 'NewPost', path: '/posts/new', component: NewPost },
  { name: 'Inbox', path: '/inbox', component: Inbox },
  { name: 'Chronometer', path: '/chronometer', component: Chronometer },
];

/* global VueRouter */
const router = new VueRouter({
  routes,
  mode: 'history',
});

const desc = {
  data: {
    internalTitle: 'Home',

    msgs: [],
    chrono: {
      day: null,
      hour: null,
      min: null,
      sec: null,
      subsec: null,
    },
    chronoDesc: null,
  },

  router,

  methods: {
    async updateMsg(/* muted = false */) {
      // TODO: implement notification
      this.msgs = await request('/api/msg');
    },

    async updateChrono() {
      this.chronoDesc = await request('/api/chrono');
    },

    startLoop() {
      this.updateMsg(true);
      this.updateChrono();

      setInterval(() => {
        this.updateMsg();
        this.updateChrono();
      }, MSG_POLL_INTERVAL);

      this.tick();
    },

    tick() {
      const now = Date.now() / 1000;
      if(!this.chronoDesc) {
        requestAnimationFrame(() => this.tick());
        return;
      }

      const diff = now - this.chronoDesc.real;
      const converted = diff * this.chronoDesc.ratio + this.chronoDesc.anchor;

      /* global moment */
      const m = moment(converted * 1000);
      this.chrono = {
        day: m.format('MM/DD'),
        hour: m.format('HH'),
        min: m.format('mm'),
        sec: m.format('ss'),
        subsec: m.format('SSS'),
      };

      requestAnimationFrame(() => this.tick());
    },
  },

  computed: {
    title: {
      get() {
        return this.internalTitle;
      },

      set(t) {
        this.internalTitle = t;
        document.title = `${t} | KLYR`;
      },
    },
  },
};

export default function bootstrap() {
  /* global VueMDCAdapter */
  Vue.use(VueMDCAdapter);

  const inst = new Vue(desc);
  inst.$mount('#app');

  inst.startLoop();
}

/* global window */
window.bootstrap = bootstrap;
