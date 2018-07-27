import { request } from '../util';

const tmpl = `
<mdc-card v-if="content" class="card-gap display-card">
  <mdc-card-header :title="content.title" :subtitle="content.tags[0]" large-title>
  </mdc-card-header>
  <mdc-card-text>
    {{ content.content }}
  </mdc-card-text>
</mdc-card>
`

export default Vue.component('display-card', {
  template: tmpl,
  props: ['id', 'to'],
  data: () => ({
    content: null,
  }),

  async created() {
    this.content = await request(`/api/posts/${this.id}/digest?maxlen=80`);
  },
});
