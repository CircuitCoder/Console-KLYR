import tmpl from './NewStep.html';
import { request } from '../util';

export default Vue.component('NewStep', {
  template: tmpl,

  data: () => ({
    title: '',

    mde: null,
  }),

  async mounted() {
    /* global SimpleMDE */
    this.mde = new SimpleMDE({ element: this.$refs.content });
  },

  methods: {
    async submit() {
      let parent = null;

      if(this.$route.name === 'ReplyStep') {
        parent = parseInt(this.$route.params.id, 10);
      }

      const resp = await request('/api/steps', 'POST', {
        parent,

        title: this.title,
        content: this.mde.value(),
        time: 0, // Ignored
      });

      if(resp.ok) {
        this.$root.$emit('show-snackbar', {
          message: 'Success',
        });

        this.$router.push({
          name: 'Step',
          params: { id: resp.id },
        });
      } else {
        this.$root.$emit('show-snackbar', {
          message: 'Failed',
        });
      }
    },
  },
});
