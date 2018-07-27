import tmpl from './EmbDigest.html';
import { request } from '../util';

export default Vue.component('emb-digest', {
  template: tmpl,

  props: ['type', 'id', 'pending'],


  data: () => ({
    content: null,
    failed: false,
  }),

  async created() {
    let url;
    if(this.type === 'post') {
      url = `/api/posts/${this.id}/digest`;
      if(this.pending) url += '?pending';
    } else if(this.type === 'step') {
      url = `/api/steps/${this.id}`;
    } else {
      throw new Error(`Unsupported type: ${this.type}`);
    }

    try {
      this.content = await request(url);
    } catch(e) {
      this.failed = true;
    }
  },
});
