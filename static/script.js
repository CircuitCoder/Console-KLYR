const desc = {
  data: {
    title: 'Console KLYR',
  },
};

export default function bootstrap() {
  /* global Vue VueMDCAdapter */
  Vue.use(VueMDCAdapter);

  const inst = new Vue(desc);
  inst.$mount('#app');
}

/* global window */
window.bootstrap = bootstrap;
