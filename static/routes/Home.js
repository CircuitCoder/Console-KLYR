import tmpl from './Home.html';
import { request } from '../util';

export default Vue.component('Home', {
  template: tmpl,
  props: ['user'],
  data: () => ({
    entries: [],
  }),

  methods: {
    async update() {
	    this.entries = [];
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
    next();
    setTimeout(() => this.update());
  },

  computed: {
    canPost() {
      console.log(this.user);
      return this.user && this.user.groups.includes('publishers');
    },
  },
});
