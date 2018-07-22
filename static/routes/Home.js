import tmpl from './Home.html';
import { request } from '../util';

export default Vue.component('Home', {
  template: tmpl,
  data: () => ({
    entries: [],
  }),

  methods: {
    async update() {
      const { tag } = this.$route.query;
      const url = tag ? `/api/posts?tag=${tag}` : '/api/posts';

      const resp = await request(url);
      this.entries = resp;
    },
  },

  beforeRouteEnter(to, from, next) {
    next(vm => vm.update());
  },

  beforeRouteUpdate(to, from, next) {
    this.update();
    next();
  },
});
