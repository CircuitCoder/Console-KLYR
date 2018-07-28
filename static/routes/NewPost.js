import tmpl from './NewPost.html';
import { request } from '../util';

export default Vue.component('NewPost', {
  template: tmpl,
  props: ['user'],

  data: () => ({
    title: '',
    category: '',

    mde: null,
  }),

  async mounted() {
    /* global SimpleMDE */
    this.mde = new SimpleMDE({ element: this.$refs.content });

    if(this.$route.name === 'EditPost') {
      const post = await request(`/api/posts/${this.$route.params.id}?pending`);
      this.mde.value(post.content);
      this.title = post.title;
      [this.category] = post.tags;
    }
  },

  methods: {
    async submit() {
      let url = '/api/posts';
      let method = 'POST';
      if(this.$route.name === 'EditPost') {
        url = `/api/posts/${this.$route.params.id}`;
        method = 'PUT';
      }
      const resp = await request(url, method, {
        title: this.title,
        author: this.user.id,
        content: this.mde.value(),
        tags: [this.category],
        time: 0, // This is ignored
      });

      if(resp.ok) {
        this.$root.$emit('show-snackbar', {
          message: 'Success',
        });
        this.$router.push({
          name: 'Post',
          params: { id: resp.id },
          query: { pending: true },
        });
      } else {
        this.$root.$emit('show-snackbar', {
          message: 'Failed',
        });
      }
    },
  },
});
