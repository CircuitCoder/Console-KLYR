import Home from './routes/Home';
import NewPost from './routes/NewPost';

const routes = [
  { name: 'Home', path: '/', component: Home },
  { name: 'NewPost', path: '/posts/new', component: NewPost },
];

/* global VueRouter */
const router = new VueRouter({
  routes,
  mode: 'history',
});

const desc = {
  data: {
    internalTitle: 'Home',
  },

  router,

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
}

/* global window */
window.bootstrap = bootstrap;
