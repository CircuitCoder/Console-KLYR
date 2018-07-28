import Home from './routes/Home';
import NewPost from './routes/NewPost';
import Inbox from './routes/Inbox';
import Chronometer from './routes/Chronometer';
import Post from './routes/Post';
import TreeView from './routes/TreeView';
import NewStep from './routes/NewStep';
import Step from './routes/Step';

import EmbDigest from './components/EmbDigest';
import PostInner from './components/PostInner';
import DisplayCard from './components/DisplayCard';

import { request } from './util';

const MSG_POLL_INTERVAL = 10 * 1000;

const routes = [
  { name: 'Home', path: '/', component: Home },
  { name: 'NewPost', path: '/posts/new', component: NewPost },
  { name: 'Inbox', path: '/inbox', component: Inbox },
  { name: 'Chronometer', path: '/chronometer', component: Chronometer },
  { name: 'Post', path: '/posts/:id', component: Post },
  { name: 'EditPost', path: '/posts/:id/edit', component: NewPost },
  { name: 'TreeView', path: '/reactor', component: TreeView },
  { name: 'NewStep', path: '/reactor/new', component: NewStep },
  { name: 'Step', path: '/reactor/:id', component: Step },
  { name: 'ReplyStep', path: '/reactor/:id/reply', component: NewStep },
];

/* global VueRouter */
const router = new VueRouter({
  routes,
  mode: 'history',
});

const desc = {
  data: {
    internalTitle: 'Console KLYR',

    msgs: [],
    chrono: {
      day: null,
      hour: null,
      min: null,
      sec: null,
      subsec: null,
    },
    chronoDesc: null,

    steps: [],
    user: null,

    login: false,
    username: '',
    password: '',
  },

  router,

  components: {
    EmbDigest,
    PostInner,
    DisplayCard,
  },

  methods: {
    async updateMsg(/* muted = false */) {
      // TODO: implement notification
      this.msgs = await request('/api/msg');
      this.msgs.sort((a, b) => b.time - a.time);
    },

    async updateChrono() {
      this.chronoDesc = await request('/api/chrono');
    },

    async updateSteps() {
      // TODO: authorize
      this.steps = await request('/api/steps/staged');
      this.steps.sort((a, b) => b - a);
    },

    startLoop() {
      this.updateMsg(true);
      this.updateChrono();
      this.updateSteps();

      setInterval(() => {
        this.updateMsg();
        this.updateChrono();
        this.updateSteps();
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

    async fetchUser() {
      this.user = null;
      try {
        this.user = await request('/api/auth');
      } catch(_) { /* Ignore */ }
    },

    async doLogin() {
      const resp = await request('/api/auth', 'POST', {
        username: this.username,
        password: this.password,
      });

      if(resp.ok) this.fetchUser();
      else {
        this.$root.$emit('show-snackbar', {
          message: 'Login failed, please contact sysadmin.',
        });
      }
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
  inst.fetchUser();
}

/* global window */
window.bootstrap = bootstrap;
